import logging

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s | %(levelname)-8s | %(message)s",
    handlers=[
        logging.StreamHandler(),
        # logging.FileHandler("quic_test.log", mode="w")
    ]
)

log = logging.getLogger()
