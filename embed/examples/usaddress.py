from sqlite_python_extensions import scalar_function, table_function, Row
import usaddress

@table_function(columns=['state','place','street','street_type','address_number','occupancy_type','occupancy_identifier','zipcode'])
def usaddresses(text):
  address = usaddress.tag(text)[0]

  yield Row(
    state=address.get('StateName'),
    place=address.get('PlaceName'),
    street=address.get('StreetName'),
    street_type=address.get('StreetNamePostType'),
    address_number=address.get('AddressNumber'),
    occupancy_type=address.get('OccupancyType'),
    occupancy_identifier=address.get('OccupancyIdentifier'),
    zipcode=address.get('ZipCode')
  )
pass