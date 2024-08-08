# logger_config.py
import logging

# Set up basic configuration for logging
logging.basicConfig(level=logging.INFO,
                    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
                    filename='process.log',
                    filemode='a')

# Optionally, create a custom logger instance (though it's not strictly necessary)
logger = logging.getLogger(__name__)
