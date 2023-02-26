up:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-metrics.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml up -d

ps:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml ps

logs:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml logs -f --tail=20

down:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml down

loaddb:
	#docker-compose -f docker-compose-postgresql.yml -f docker-compose-windmill.yml exec windmill_worker ogr2ogr -f PostgreSQL PG:'host=postgresql user=geodb password=$$PG_PASS dbname=geodb' /data/ne_extracts.gpkg
	docker-compose -f docker-compose-postgresql.yml -f docker-compose-windmill.yml exec -e PGPASSWORD windmill_worker ogr2ogr -f PostgreSQL PG:"host=postgresql user=geodb dbname=geodb" /data/ne_extracts.gpkg

resetdb:
	docker stop bbox_postgresql_1
	docker rm bbox_postgresql_1
	docker volume rm bbox_postgres-data	
