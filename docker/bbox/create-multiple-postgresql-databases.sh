#!/bin/bash

set -e
set -u

function create_user_and_database() {
	local database=$1
	echo "  Creating user and database '$database'"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
	    CREATE USER $database PASSWORD '$POSTGRES_PASSWORD';
	    CREATE DATABASE $database OWNER $database;
EOSQL
}

function setup_windmill() {
		psql windmill -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" -f /docker-entrypoint-initdb.d/init-windmill-as-superuser.sql
		psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
			GRANT windmill_admin TO windmill;
			GRANT windmill_user TO windmill;
EOSQL
}

if [ -n "$POSTGRES_MULTIPLE_DATABASES" ]; then
	echo "Multiple database creation requested: $POSTGRES_MULTIPLE_DATABASES"
	for db in $(echo $POSTGRES_MULTIPLE_DATABASES | tr ',' ' '); do
		create_user_and_database $db
		if [ "$db" == "windmill" ]; then
			setup_windmill
		fi
	done
	echo "Multiple databases created"
fi
