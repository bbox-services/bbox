compose_files := -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-metrics.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose.yml

up:
	docker-compose $(compose_files) up -d

ps:
	docker-compose $(compose_files) ps

logs:
	docker-compose $(compose_files) logs -f --tail=20 $(c)

restart:
	docker-compose $(compose_files) restart $(c)

down:
	docker-compose $(compose_files) down

loaddb:
	#docker-compose -f docker-compose-postgresql.yml -f docker-compose-windmill.yml exec windmill_worker ogr2ogr -f PostgreSQL PG:'host=postgresql user=geodb password=$$PG_PASS dbname=geodb' /data/ne_extracts.gpkg
	docker-compose -f docker-compose-postgresql.yml -f docker-compose-windmill.yml exec -e PGPASSWORD windmill_worker ogr2ogr -f PostgreSQL PG:"host=postgresql user=geodb dbname=geodb" /data/ne_extracts.gpkg

resetdb:
	docker stop bbox_postgresql_1
	docker rm bbox_postgresql_1
	docker volume rm bbox_postgres-data	

teamengine:
	docker-compose -f docker-compose-teamengine.yml up -d
	# open http://localhost:8081/teamengine/
	# Login with ogctest / ogctest
	# Landing page location: http://bbox:8080/ogcapi/
