from makepie import *

@default()
def run():
	sh("cargo run")

def build():
	sh("cargo build")
