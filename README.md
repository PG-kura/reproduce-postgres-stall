# Reproduce postgres stall

> **Warning**
The stalls described in this README may be my misunderstanding.


## Overview
We are facing postgres queries stalling.

I've done some research and still don't know the cause.
This repository was created to provide highly reproducible source code.

## 2 way to use

### Requirements
* docker-compose
  
### Commands
* Use psql
    ```bash
    $ ROW_COUNT_LOG10=5 docker-compose up --abort-on-container-exit psql-client
    ```
* Use tokio-postgres
    ```bash
    $ ROW_COUNT_LOG10=5 docker-compose up --abort-on-container-exit my-client
    ```

Environment variable `ROW_COUNT_LOG10` epecifies row count of response. This variable indicates what power 10 should be raised to. For example 1 means 10 rows, 3 means 1,000 rows.