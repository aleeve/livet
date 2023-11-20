create table if not exists
musicians (
  id integer primary key not null,
  name text
);

create table if not exists
band (
  id integer not null,
  name text,
  member integer,
  primary key (id),
  foreign key (member)
    refrences musicians (id)
);
