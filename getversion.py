import configparser

config = configparser.ConfigParser()
config.read('Cargo.toml')
print(config['package']['version'].strip(r"\""))