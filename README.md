# Micro-Service Interface Protocol

This repository provides a custom protocol for communication between micro-services and servers. It aims to facilitate seamless interaction and data exchange in a distributed system architecture.

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

...

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

...

### msg_kind (2 bytes)

| Valeur | Signification |
| -----: | ------------- |
|      1 | EVENT         |
|      2 | COMMAND       |
|      3 | STATE         |
|      4 | LOG           |
|      5 | METRIC        |

...
