# logger_config.py
import logging
from os.path import join

# Set up basic configuration for logging
def configure_logger(path):
    filename = join(path, '_nextsyncengine_', 'process.log') if path != "" else 'process.log'
    logging.basicConfig(level=logging.WARNING,
                    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
                    filename=filename,
                    filemode='a')

