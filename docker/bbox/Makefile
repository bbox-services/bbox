profile ?= default

up:
	docker compose --profile $(profile) up -d

ps:
	docker compose --profile $(profile) ps

logs:
	docker compose --profile $(profile) logs -f --tail=20 $(c)

restart:
	docker compose --profile $(profile) restart $(c)

down:
	docker compose --profile $(profile) down

loaddb:
	#docker compose --profile processes run windmill_worker ogr2ogr -f PostgreSQL PG:'host=postgresql user=geodb password=$$PG_PASS dbname=geodb' /data/ne_extracts.gpkg
	docker compose --profile processes run windmill_worker ogr2ogr -f PostgreSQL PG:"host=postgresql user=geodb dbname=geodb" /data/ne_extracts.gpkg

resetdb:
	docker stop bbox_postgresql_1
	docker rm bbox_postgresql_1
	docker volume rm bbox_postgres-data	

teamengine:
	docker compose up -d teamengine
	# open http://localhost:8081/teamengine/
	# Login with ogctest / ogctest
	# Landing page location: http://bbox:8080/
