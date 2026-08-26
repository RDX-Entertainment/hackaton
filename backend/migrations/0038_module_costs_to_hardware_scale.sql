-- El consumo de los modulos, en las unidades del hardware que lo tiene que aguantar.
--
-- 0036 dio a cada modulo un coste de CPU, RAM y red en numeros pequenos —de 1 a 10—, como si
-- fueran atributos. No lo son: `operations.start` los compara contra `hardware.cpu/ram/net`,
-- que estan en la escala del catalogo de piezas. Ahi una maquina de arranque trae CPU 1000,
-- RAM 512 y red **1**, porque la NET1 es la pieza mas baja que hay.
--
-- Con la escala vieja el consumo de CPU y RAM era ruido —el 1% de la maquina, y la duracion
-- no se movia— y el de red era imposible: pedia 5 a quien tiene 1. Ninguna operacion podia
-- empezar.
--
-- Aqui se reescriben en la escala buena. La red es la que decide, y es barata a proposito:
-- la escalera es 1, 2, 4, 10, 25... y un ataque con dos modulos pide 5, o sea una MEGA10 de
-- 250 creditos. Subir de pieza es una decision de verdad y no un muro.
UPDATE research_modules SET cpu_delta =  40, ram_delta =  64, net_delta = 1 WHERE code = 'ssh_obfuscation';
UPDATE research_modules SET cpu_delta = 200, ram_delta = 128, net_delta = 3 WHERE code = 'ssh_bruteforce';
UPDATE research_modules SET cpu_delta =  60, ram_delta =  48, net_delta = 1 WHERE code = 'ftp_compatibility';
UPDATE research_modules SET cpu_delta = 160, ram_delta =  96, net_delta = 1 WHERE code = 'wordlist_engine';
UPDATE research_modules SET cpu_delta =  80, ram_delta = 112, net_delta = 2 WHERE code = 'virus_polymorph';

-- Los perfiles ya compilados se quedan como estan. Son copias materializadas: lo que valia
-- un programa el dia que se compilo es lo que vale, y una tabla de costes nueva no reescribe
-- lo que alguien ya tiene en el disco. Recompilar el blueprint da la version nueva.
