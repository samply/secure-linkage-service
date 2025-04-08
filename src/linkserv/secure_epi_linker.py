import base64
import binascii

import numpy as np
from Crypto import Random
from Crypto.Cipher import AES
from Crypto.Util import Counter


class AESCipher:
    def __init__(self, key, key_length=32):
        self.key = key
        self.key_length = key_length
        assert len(self.key) == self.key_length

    # Standard aes in counter mode, IV gets prepended. Optional base64 encoding
    def encrypt(self, plain_text, key, need_to_provide_b64_encoded_data=True):
        # Just to make sure we are using the right key.
        assert key == self.key

        iv = Random.new().read(AES.block_size)
        iv_int = int(binascii.hexlify(iv), 16)
        ctr = Counter.new(AES.block_size * 8, initial_value=iv_int)
        cipher = AES.new(key, AES.MODE_CTR, counter=ctr)
        encrypted_text = iv + cipher.encrypt(plain_text)
        if need_to_provide_b64_encoded_data:
            return base64.b64encode(encrypted_text)
        return encrypted_text

    def decrypt(self, encrypted_text, key, is_encrypted_text_b64_encoded=True):
        # Just to make sure we are using the right key.
        assert key == self.key

        # Probably the whole logic of this class is flawed.
        #
        # Using base64 here is critical.
        if is_encrypted_text_b64_encoded:
            encrypted_text = base64.b64decode(encrypted_text)
        iv_int = int(binascii.hexlify(encrypted_text[:16]), 16)
        ctr = Counter.new(AES.block_size * 8, initial_value=iv_int)
        cipher = AES.new(key, AES.MODE_CTR, counter=ctr)
        return cipher.decrypt(encrypted_text[16:])


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
        for i in range(0, n):
            result[i] = self.encrypt(
                b'0' + bytes(self.idcounter) + self.pad * b'0', self.key, True)
            self.idcounter += 1
        return result.tolist()

    def validate_id(self, encrypted_linkage_id, is_linkage_id_b64_encoded=True):
        m = self.cipher.decrypt(encrypted_linkage_id, self.key, is_linkage_id_b64_encoded)
        return m[-self.pad:] == self.pad * b'0'

    def decrypt(self, encrypted_text, is_encrypted_text_b64_encoded=True):
        return self.cipher.decrypt(str.encode(encrypted_text), self.key, is_encrypted_text_b64_encoded)

    def encrypt(self, plain_text, need_to_provide_b64_encoded_data=True):
        return self.cipher.encrypt(plain_text, self.key, need_to_provide_b64_encoded_data)
