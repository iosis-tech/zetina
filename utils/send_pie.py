import grpc
import delegator_pb2_grpc
import delegator_pb2

def send_file_to_server():
    # Open the file and read its contents
    with open("cairo0_fibonacci_pie.zip", 'rb') as file:
        file_content = file.read()

    # Create gRPC channel and stub
    with grpc.insecure_channel('localhost:50051') as channel:
        stub = delegator_pb2_grpc.DelegatorServiceStub(channel)

        # Define generator function for streaming request
        def request_generator():
            for i in range(2):
                yield delegator_pb2.DelegateRequest(cairo_pie=file_content)

        try:
            # Make the RPC call with streaming request
            responses = stub.Delegator(request_generator())

            # Iterate over responses (if needed)
            for response in responses:
                print(response)

        except grpc.RpcError as e:
            # Handle gRPC errors
            print(f"Error occurred during RPC: {e}")

            # Access detailed information about the RPC status
            status_code = e.code()
            details = e.details()
            debug_error_string = e.debug_error_string()

            print(f"RPC Error Details:")
            print(f"Status Code: {status_code}")
            print(f"Details: {details}")
            print(f"Debug Error String: {debug_error_string}")

if __name__ == '__main__':
    send_file_to_server()
