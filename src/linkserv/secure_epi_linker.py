import base64
import binascii
from Crypto.Cipher import AES
from Crypto.Util import Counter
from Crypto import Random
import numpy as np


class AESCipher:
    def __init__(self, key, key_length=32):
        self.key = key
        self.key_length = key_length
        assert len(self.key) == self.key_length

    # Standard aes in counter mode, IV gets prepended. Optional base64 encoding
    def encrypt(self, plain, key, b64=True):
        # Just to make sure we are using the right key.
        assert key == self.key

        iv = Random.new().read(AES.block_size)
        iv_int = int(binascii.hexlify(iv), 16)
        ctr = Counter.new(AES.block_size * 8, initial_value=iv_int)
        cipher = AES.new(key, AES.MODE_CTR, counter=ctr)
        if b64:
            return base64.b64encode(iv + cipher.encrypt(plain))
        return iv + cipher.encrypt(plain)

    def decrypt(self, enc, key, b64=True):
        # Just to make sure we are using the right key.
        assert key == self.key

        # Probably the whole logic of this class is flawed.
        #
        # Using base64 here is critical.
        if b64:
            enc = base64.b64decode(enc)
        iv_int = int(binascii.hexlify(enc[:16]), 16)
        ctr = Counter.new(AES.block_size * 8, initial_value=iv_int)
        cipher = AES.new(key, AES.MODE_CTR, counter=ctr)
        return cipher.decrypt(enc[16:])


class SecureEpilinker:
    def __init__(self, uid, key, pad, idcounter=0):
        self.uid = uid
        self.key = key
        self.cipher = AESCipher(key)
        self.idcounter = idcounter
        self.pad = pad

    # The fresh IDs are comprised of:
    # * a 0-bit (tmatch in the linkageresult),
    # * an incrementing counter, and
    # * a specific zero padding for each site.
    #
    # This bit-string is then encrypted using AES.
    def generate_id(self, n=1):
        # TODO: Ask Tobias whether
        # was NumPy used here only for serialization reasons?
        result = np.empty(n, dtype='S64')
        b64 = True
        for i in range(0, n):
            result[i] = self.cipher.encrypt(
                b'0' + bytes(self.idcounter) + self.pad * b'0', self.key, b64)
            self.idcounter += 1
        return result.tolist()

    def validate_id(self, lid, b64=True):
        m = self.cipher.decrypt(lid, self.key, b64)
        return m[-self.pad:] == self.pad * b'0'

    def decrypt(self, enc, b64=True):
        return self.cipher.decrypt(str.encode(enc), self.key, b64)

    def encrypt(self, msg, b64=True):
        return self.cipher.encrypt(msg, self.key, b64)
