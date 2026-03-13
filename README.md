# Micro-Service Interface Protocol

This repository provides a custom protocol for communication between micro-services and servers. It aims to facilitate seamless interaction and data exchange in a distributed system architecture.

## Features

- 🚀 **High Performance**: Designed for low latency and high throughput communication
- 📊 **Structured Messaging**: Clear message format with a fixed-size header and variable-length payload
- 🔄 **Session Resumption**: Automatically resume sessions after disconnections
- 📡 **Robust Communication**: Reliable message delivery with ACK and ERROR handling
- 🧩 **Extensible**: Easily extendable protocol to accommodate future features
- 📚 **Client Libraries**: Available client libraries in TypeScript, Python, and Rust for easy integration
- 🐳 **Docker Support**: Ready-to-use Docker images for quick deployment

```dockerfile
docker pull doctorpok/mip:latest
```

## Client-Server Communication

The protocol defines a structured format for messages exchanged between clients and servers. Each message consists of a fixed-size header followed by a variable-length payload. The header contains essential metadata about the message, while the payload carries the actual data being transmitted.

|     Header     |    Payload     |
| -------------- | -------------- |
|    24 bytes    |   N bytes      |

## Header binaire (24 bytes)

| Offset | Taille | Champ       | Description           |
| -----: | -----: | ----------- | --------------------- |
|      0 |      4 | magic       | `"MSIP"` (0x4D425553) |
|      4 |      1 | version     | Version protocole (1) |
|      5 |      1 | flags       | Bits de contrôle      |
|      6 |      2 | frame_type  | Type de frame         |
|      8 |      2 | msg_kind    | Type applicatif       |
|     10 |      2 | reserved    | Alignement / futur    |
|     12 |      4 | payload_len | Taille du payload     |
|     16 |      8 | msg_id      | ID message unique     |

### Flags (1 byte)

| Bit | Nom          | Description       |
| --: | ------------ | ----------------- |
|   0 | ACK_REQUIRED | ACK attendu       |
|   1 | COMPRESSED   | Payload compressé |
|   2 | URGENT       | Priorité élevée   |
|   3 | RESERVED     | futur             |

### Types de frame (2 bytes)

| Valeur | Nom         |
| -----: | ----------- |
| 0x0001 | HELLO       |
| 0x0002 | SUBSCRIBE   |
| 0x0003 | UNSUBSCRIBE |
| 0x0004 | PUBLISH     |
| 0x0005 | EVENT       |
| 0x0006 | ACK         |
| 0x0007 | ERROR       |
| 0x0008 | PING        |
| 0x0009 | PONG        |
| 0x000A | CLOSE       |

### msg_kind (2 bytes)

| Valeur | Signification |
| -----: | ------------- |
|      1 | EVENT         |
|      2 | COMMAND       |
|      3 | STATE         |
|      4 | LOG           |
|      5 | METRIC        |

## Payload (N bytes)

The payload contains the actual data being transmitted. Its structure depends on the frame type and message kind. For example, a PUBLISH frame with msg_kind = EVENT might contain a JSON object representing an event, while a SUBSCRIBE frame might contain a list of topics.

## Client Library

A client library is provided to facilitate the implementation of micro-services that communicate using this protocol. The library abstracts away the details of message formatting and parsing, allowing developers to focus on the application logic.

[TypeScript Client Library](https://github.com/DoctorPok42/MIP-Clients/tree/main/mip-client-ts#readme)  
[Python Client Library](https://github.com/DoctorPok42/MIP-Clients/tree/main/mip-client-python#readme)  
[Rust Client Library](https://github.com/DoctorPok42/MIP-Clients/tree/main/mip-client-rust#readme)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
