Minimal Available System Test on Google Cloud
---

### VPC for Serverless

**Update**: The easiest way to do it is to deploy Cloud SQl and Compute Engine's private IP section as default and no need to create an individual VPC.

---

### Cloud SQL

+ Create a postgres database in GCP console.

*By the way, the *default config is 1 vcpu, 3.75G RAM and 20G SSD*.

+ Edit instance, enable private IP connection

*Public could be disabled later for security*.

+ Test: a) Install `postgresql-client` and `gcloud sdk` on local machine and connect to database in gcloud sdk tool:

    gcloud sql connect instance-name --user=postgres

Or b) Install `postgresql-client` on Compute Engine and connect to database:

    sudo apt install postgresql-client
    psql -h <privite ip address> -U username

Now create database, table, and populate some data into it.

Build rust program in release mode and upload it to a cloud instance for test.

---

### Secret Manager

+ Store the database password to Secreat manager.

+ IAM -> Service account -> Create service account -> Give secret access permission to a role.

+ Create an Compute Engine instance of this role. Now it has permission to retrieve secret by:

    curl "https://secretmanager.googleapis.com/v1/projects/`project-id`/secrets/`secret-id`/versions/`version-id`:access" \
        --request "GET" \
        --header "authorization: Bearer $(gcloud auth print-access-token)" \
        --header "content-type: application/json" \
        --header "x-goog-user-project: `project-id`"

**Note**: the response needs to be decoded by base64.

---

### Local test of Postgres and Rust code communication

Start a postgres container from docker-compose.yml

    version: '3'
    services:
    db:
        image: postgres:latest
        container_name: db
        restart: "no"
        ports:
        - "5432:5432"
        environment:
        - POSTGRES_USER=admin
        - POSTGRES_PASSWORD=1a2b3c
        - POSTGRES_DB=test
        volumes:
        - ./pgdata:/var/lib/postgresql/data
        - ./pginit:/docker-entrypoint-initdb.d/


Then step into it by :

    docker exec -it db psql -U admin test

or:

    docker exec -it bash
    psql -U admin test

