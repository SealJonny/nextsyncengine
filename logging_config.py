# logger_config.py
import logging

# Set up basic configuration for logging
logging.basicConfig(level=logging.WARNING,
                    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
                    filename='process.log',
                    filemode='a')
