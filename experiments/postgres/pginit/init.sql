CREATE TABLE customer (
    id serial primary key,
    email varchar(30) not null,
    password varchar(64) not null,
    nickname varchar(16) default 'Anonymous'
);

INSERT INTO customer (email, password, nickname) VALUES ('a@b.com', '123456', 'Cheng');
