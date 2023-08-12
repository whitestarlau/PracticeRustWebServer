drop table if exists goods_summary;
create table goods_summary (
       id serial primary key,
       name varchar(140),
       image varchar(140)
);

insert into goods_summary (name , image) values('book1','');
insert into goods_summary (name , image) values('book2','');
insert into goods_summary (name , image) values('book3','');


drop table if exists goods_detail;
create table goods_detail (
       id serial primary key,
       name varchar(140),
       image varchar(140),
       des varchar(140),
       unit_price INT not NULL
);

insert into goods_detail (name ,image, des, unit_price) values('book1','', 'This is test des.' ,200);
insert into goods_detail (name ,image, des, unit_price) values('book2','', 'This is test des.' ,200);
insert into goods_detail (name ,image, des, unit_price) values('book3','', 'This is test des.' ,200);
