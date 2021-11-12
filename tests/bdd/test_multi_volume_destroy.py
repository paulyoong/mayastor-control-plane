import os
import subprocess

import pytest
import docker

from common.nvme import nvme_connect, nvme_discover, nvme_disconnect_all
from common.deployer import Deployer
from common.apiclient import ApiClient
from common.docker import Docker
from openapi.model.create_pool_body import CreatePoolBody
from openapi.model.create_volume_body import CreateVolumeBody
from openapi.model.volume_policy import VolumePolicy
from openapi.model.protocol import Protocol
from common.fio import Fio

VOLUME_UUID = "5cd5378e-3f05-47f1-a830-a0f5873a144X"
VOLUME_SIZE = 10485761
NUM_VOLUME_REPLICAS = 2
MAYASTOR_1 = "mayastor-1"
MAYASTOR_2 = "mayastor-2"
POOL_DISK1 = "disk1.img"
POOL1_UUID = "4cc6ee64-7232-497d-a26f-38284a444980"
POOL_DISK2 = "disk2.img"
POOL2_UUID = "4cc6ee64-7232-497d-a26f-38284a444990"
FIO_RUNTIME_SECS = 20


@pytest.fixture(scope="function")
def create_pool_disk_images():
    # When starting Mayastor instances with the deployer a bind mount is created from /tmp to
    # /host/tmp, so create disk images in /tmp
    for disk in [POOL_DISK1, POOL_DISK2]:
        path = "/tmp/{}".format(disk)
        with open(path, "w") as file:
            file.truncate(1 * 1024 * 1024 * 1024)

    yield

    for disk in [POOL_DISK1, POOL_DISK2]:
        path = "/tmp/{}".format(disk)
        if os.path.exists(path):
            os.remove(path)


@pytest.fixture(autouse=True)
def init(create_pool_disk_images):
    # Shorten the reconcile periods and cache period to speed up the tests.
    Deployer.start_with_args(
        [
            "-j",
            "-m=2",
            "-w=10s",
            "--reconcile-idle-period=500ms",
            "--reconcile-period=500ms",
            "--cache-period=1s",
        ]
    )

    # Create pools
    ApiClient.pools_api().put_node_pool(
        MAYASTOR_1,
        POOL1_UUID,
        CreatePoolBody(["aio:///host/tmp/{}".format(POOL_DISK1)]),
    )
    ApiClient.pools_api().put_node_pool(
        MAYASTOR_2,
        POOL2_UUID,
        CreatePoolBody(["aio:///host/tmp/{}".format(POOL_DISK2)]),
    )

    yield
    # Deployer.stop()


def check_all_mayastor_running():
    docker_client = docker.from_env()
    try:
        mayastors = docker_client.containers.list(
            all=True, filters={"name": "mayastor"}
        )
    except docker.errors.NotFound:
        raise Exception("No Mayastor instances")

    for mayastor in mayastors:
        try:
            Docker.check_container_running(mayastor.attrs["Name"])
        except:
            return False

    return True


# Create and published the desired number of volumes.
# Return the target URIs and volume UUIDs
def create_and_publish_volumes(num_volumes):
    target_uris = []
    volume_uuids = []
    for i in range(0, num_volumes):
        uuid = VOLUME_UUID.replace("X", str(i))
        volume_uuids.append(uuid)
        request = CreateVolumeBody(
            VolumePolicy(False), NUM_VOLUME_REPLICAS, VOLUME_SIZE
        )
        ApiClient.volumes_api().put_volume(uuid, request)
        volume = ApiClient.volumes_api().put_volume_target(
            uuid, MAYASTOR_1, Protocol("nvmf")
        )
        target_uris.append(volume["state"]["target"]["deviceUri"])
    return target_uris, volume_uuids


# Destroy all volumes with the given UUIDs
def destroy_volumes(uuids):
    for volume in uuids:
        ApiClient.volumes_api().del_volume(volume)


# Discover and connect to all the NVMe URIs
def nvme_discover_and_connect(uris):
    nvme_devices = []
    for uri in uris:
        nvme_discover(uri)
        nvme_devices.append(nvme_connect(uri))
    return nvme_devices


# Run a fio process per device
def run_fio_processes(devices):
    fio_processes = []
    for device in devices:
        print("Run fio against NVMe device {}".format(device))
        fio = Fio("job", "randread", device, runtime=FIO_RUNTIME_SECS).build()
        fio_processes.append(subprocess.Popen(fio, shell=True))
    return fio_processes


# Wait for all fio processes to complete
def wait_for_fio_processes(processes):
    for process in processes:
        try:
            code = process.wait(timeout=FIO_RUNTIME_SECS * 2)
        except subprocess.TimeoutExpired:
            assert False, "FIO timed out"
        assert code == 0, "FIO failed, exit code: %d" % code


def test_mutli_volume():
    num_volumes = 10
    while True:
        target_uris, volume_uuids = create_and_publish_volumes(num_volumes)
        nvme_devices = nvme_discover_and_connect(target_uris)
        fio_processes = run_fio_processes(nvme_devices)
        wait_for_fio_processes(fio_processes)
        destroy_volumes(volume_uuids)

        # Check there are no volumes remaining
        assert len(ApiClient.volumes_api().get_volumes()) == 0

        # Check all Mayastor instances are still running
        all_running = check_all_mayastor_running()

        # Disconnect all NVMe devices
        nvme_disconnect_all()

        # If we find that Mayastor has stopped running, exit the test
        if not all_running:
            print("Some Mayastor instances not running")
            break
