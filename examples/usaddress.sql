
-- sqlite3x :memory: '.read examples/usaddress.sql'
.load target/debug/libpy0

.timer on

insert into py_functions(name, function)
  select 
    'parse_address' as name, 
    py_function_from_module('
import usaddress

def parse_address(address):
  return usaddress.tag(address)[0]
    ',
    'parse_address'
  );

.mode box
.header on
with addresses as (
  select
    value as address,
    parse_address(value) as parsed
  from json_each('[
    "123 Main St. Suite 100 Chicago, IL",
    "11604 Whittier Blvd, Whittier, CA 90601",
    "5005 Paramount Blvd, Pico Rivera, CA 90660",
    "5600 Whittier Blvd, Commerce, CA 90022"
  ]')
)
select 
  address,
  py_call_method(parsed, 'get', 'StateName') as state,
  py_call_method(parsed, 'get', 'PlaceName') as place,
  py_call_method(parsed, 'get', 'StreetName') as street,
  py_call_method(parsed, 'get', 'StreetNamePostType') as street_type,
  py_call_method(parsed, 'get', 'AddressNumber') as address_number,
  py_call_method(parsed, 'get', 'OccupancyType') as occupancy_type,
  py_call_method(parsed, 'get', 'OccupancyIdentifier') as occupancy_identifier,
  py_call_method(parsed, 'get', 'ZipCode') as zipcode
from addresses;