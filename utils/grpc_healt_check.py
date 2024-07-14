import logging
from time import sleep

import grpc
from grpc_health.v1 import health_pb2
from grpc_health.v1 import health_pb2_grpc

def health_check_call(stub: health_pb2_grpc.HealthStub):
    request = health_pb2.HealthCheckRequest()
    resp = stub.Check(request)
    if resp.status == health_pb2.HealthCheckResponse.SERVING:
        print("server is serving")
    elif resp.status == health_pb2.HealthCheckResponse.NOT_SERVING:
        print("server stopped serving")


def run():
    with grpc.insecure_channel("localhost:50052") as channel:
        health_stub = health_pb2_grpc.HealthStub(channel)

        # Check health status every 1 second for 30 seconds
        for _ in range(30):
            health_check_call(health_stub)


if __name__ == "__main__":
    logging.basicConfig()
    run()