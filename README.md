# pp

grpcurl -proto proto/network.proto -d '{"host": "[::1]", "port": 3001}' -plaintext localhost:50000 network.grpc.NetworkService.Initiate

grpcurl -proto proto/network.proto \
 -d '{"public_key": "7be42b204c00d965c891848c06180f504dbf0e26f8bbae647d8bf366dacea17a" }' \
 -plaintext localhost:50001 network.grpc.NetworkService.Recv

grpcurl -proto proto/network.proto \
 -d '{"public_key": "649293486b0d3af1f90243021453dcb7dbbbd9fd3a54c373eaca02d230aa3154", "data": "0101"}' \
 -plaintext localhost:50000 network.grpc.NetworkService.Send
