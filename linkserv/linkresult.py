import base64
import binascii
import sys
import operator
import simplejson as json
from linkserv.secureepilinker import SecureEpilinker
from flask import make_response

class LinkageResult:
    def __init__(self, client, server):
        self.client = client
        self.client_matches = []
        self.client_tmatches = []
        self.client_idxs = []
        self.server = server
        self.server_matches = []
        self.server_tmatches = []
        self.server_idxs = []
        self.client_data = False
        self.server_data = False
        self.server_ids = []

    def clear(self):
        self.client_data = False
        self.server_data = False
        self.server_ids = []
        self.client_matches = []
        self.client_tmatches = []
        self.client_idxs = []
        self.server_matches = []
        self.server_tmatches = []
        self.server_idxs = []

    def set_share(self, json_data):
        # data = json.load(json_data)
        data = json_data
        if (data['role'] == 'server'):
            for res in data['result']:
                print("Record: ", res)
                self.server_matches.append(res['match'])
                self.server_tmatches.append(res['tentativeMatch'])
                self.server_idxs.append(res['bestIndex'])
            self.server_data = True
            self.server_ids = data['ids']
            print("Server Matches: ", self.server_matches)
        elif (data['role'] == 'client'):
            for res in data['result']:
                self.client_matches.append(res['match'])
                self.client_tmatches.append(res['tentativeMatch'])
                self.client_idxs.append(res['bestIndex'])
            self.client_data = True
            print("Client Matches: ", self.client_matches)
        else:
            return make_response('Invalid role', 400)

    def get_linkage_result(self):
        # All inputs set? And all lists well formed?
        assert self.server_data and self.client_data is True
        assert len(self.server_matches) == len(self.client_matches)
        assert len(self.server_tmatches) == len(self.client_tmatches)
        assert len(self.server_idxs) == len(self.client_idxs)
        assert len(self.server_matches) == len(self.server_tmatches)
        assert len(self.server_matches) == len(self.server_idxs)
        # Compute clear values
        ids = []
        for cmatch, smatch, ctmatch, stmatch, cidx, sidx in zip(self.client_matches, self.server_matches, self.client_tmatches, self.server_tmatches, self.client_idxs, self.server_idxs):
            match = cmatch ^ smatch
            tmatch = ctmatch ^ stmatch
            bestInd = cidx ^ sidx
            if match or tmatch:
                # Compute best ID
                # Decrypt ID with server key and encrypt with client key prepend
                # tmatch bit
                # print(self.server.validate_id(bestId,base64=False))
                client_linkage_id = self.client.encrypt(bytes(tmatch)+self.server.decrypt(self.server_ids[bestInd],False))
                single = len(self.client_matches) == 1
                if single:
                    return json.dumps({'result': {'linkageId': client_linkage_id}})
                else:
                    ids.append({'linkageId': client_linkage_id})
            else:
                single = len(self.client_matches) == 1
                self.clear()
                if single:
                    return json.dumps({'result': {'linkageId': self.client.generate_id()}})
                else:
                    ids.append({'linkageId': self.client.generate_id()})
        self.clear()
        return json.dumps({'result':ids})

    def is_ready(self):
        return self.server_data and self.client_data
