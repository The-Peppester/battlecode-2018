from trueskill import Rating, quality_1vs1, rate_1vs1
import psycopg2
import json
from time import sleep
import threading
import random

pg = psycopg2.connect("dbname='battlecode' user='battlecode' host='" + os.environ["DB_HOST"] + "' password='" + os.environ["DB_PASS"] + "'")
cur = pg.cursor()
print("Connected to postgres.")

QUEUED_MATCHES = {}
DB_LOCK = False
QUEUE_RANGE = 50
MAPS = ['socket.bc18map']

def update_loop():
    while True:
        sleep(1)
        while DB_LOCK:
            sleep(0.1)
        DB_LOCK = True
        cur.execute("SELECT (id, status) FROM match_kube WHERE (id=2 or id=3) and id IN %s", (QUEUED_MATCHES.keys(),))
        games = cur.fetchall()
        DB_LOCK = False

        for game in games:
            red_player = QUEUED_MATCHES[game[0]]['red']
            blue_player = QUEUED_MATCHES[game[0]]['red']

            rs = Rating(mu=red['mu'],simga=red['sigma'])
            bs = Rating(mu=blue['mu'],simga=blue['sigma'])

            rn, bn = rate_1vs1(rs, bs) if game[1] == 2 else rate_1vs1(bs, rs)

            while DB_LOCK:
                sleep(0.1)
            DB_LOCK = True
            cur.execute("UPDATE battlecode_users SET (mu,sigma)=(%s,%s) WHERE id=%s",(rn.mu,rn.sigma,red_player['id']))
            pg.commit()
            cur.execute("UPDATE battlecode_users SET (mu,sigma)=(%s,%s) WHERE id=%s",(bn.mu,bn.sigma,blue_player['id']))
            pg.commit()
            DB_LOCK = False

def softmax(x, T=1):
    s = sum([exp(n/T) for n in x])
    return [exp(n/T)/s for n in x]

threading.Thread(target=update_loop).start()

while True:
    sleep(60)
    while DB_LOCK:
        sleep(0.1)
    DB_LOCK = True

    cur.execute("SELECT (id, mu, sigma, upload_s3_key) FROM battlecode_teams ORDER BY mu DESC")
    users = cur.fetchall()

    DB_LOCK = False

    matches = []
    for index, user in enumerate(users):
        upper_bound = index-QUEUE_RANGE if index > QUEUE_RANGE else 0
        lower_bound = index+QUEUE_RANGE if index < len(users)-QUEUE_RANGE else len(users) #not inclusive

        options = users[upper_bound:lower_bound]
        qualities = []
        softmaxes = []
        for option in options:
            if option != user:
                qualities.append({'red':{'id':user[0],'mu':user[1],'sigma':user[2],'s3':user[3]},
                                 {'blue':{'id':option[0],'mu':option[1],'sigma':option[2],'s3':option[3]}})
                qualities[-1]['qual'] = quality_1vs1(Rating(mu=user[1],simga=user[2]),
                                                     Rating(mu=option[1],simga=option[2])))
                softmaxes.append(qualities[-1]['qual'])

        softmaxes = softmax(softmaxes)
        sample, action = random.random(), -1
        while sample >= 0:
            action += 1
            sample -= qualities[action]['prob']
        matches.append(qualities[action])

    while DB_LOCK:
        sleep(0.1)
    DB_LOCK = True
    for match in matches:
        random_map = random.choice(MAPS)
        cur.execute("INSERT INTO match_kube (red_key, blue_key, map, status) VALUES (%s,%s,%s,0) RETURNING id",(match['red']['s3'],match['blue']['s3'],random_map))
        pg.commit()
        QUEUED_MATCHES[cur.fetchone()[0]] = match
    DB_LOCK = False
