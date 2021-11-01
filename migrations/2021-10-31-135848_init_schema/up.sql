-- Your SQL goes here
CREATE TABLE revenues
(
    day   DATE PRIMARY KEY NOT NULL,
    value REAL NOT NULL
);
CREATE TABLE net_profits
(
    day   DATE  PRIMARY KEY NOT NULL,
    value REAL NOT NULL
);