from flask import make_response
import json


class LinkageResult:
    def __init__(self, client, server):
        self.client = client
        self.server = server
        self.reset();

    def reset(self):
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
        data = json_data
        if (data['role'] == 'server'):
            print("Server Data: ", data)
            for res in data['result']:
                self.server_matches.append(res['match'])
                self.server_tmatches.append(res['tentativeMatch'])
                self.server_idxs.append(res['bestIndex'])
            self.server_data = True
            self.server_ids = data['ids']
        elif (data['role'] == 'client'):
            print("Client Data: ", data)
            for res in data['result']:
                self.client_matches.append(res['match'])
                self.client_tmatches.append(res['tentativeMatch'])
                self.client_idxs.append(res['bestIndex'])
            self.client_data = True
        else:
            return make_response('Invalid role', 400)

    def is_ready(self):
        return self.server_data and self.client_data

    @staticmethod
    def get_linkage_result(client_data, server_data, client, server):
        """
        Calculate linkage result from client and server data in a completely stateless manner.

        Args:
            client_data: dict with client matching results containing arrays of match, tentativeMatch, and bestIndex.
            server_data: dict with server matching results and ids.
            client: client object used for encryption and id generation.
            server: server object used for decryption.

        Returns:
            JSON string containing the linkage result
        """
        # Extract server data
        server_matches = []
        server_tmatches = []
        server_idxs = []
        for res in server_data['result']:
            server_matches.append(res['match'])
            server_tmatches.append(res['tentativeMatch'])
            server_idxs.append(res['bestIndex'])
        server_ids = server_data['ids']

        # Extract client data
        client_matches = []
        client_tmatches = []
        client_idxs = []
        for res in client_data['result']:
            client_matches.append(res['match'])
            client_tmatches.append(res['tentativeMatch'])
            client_idxs.append(res['bestIndex'])

        # Validate inputs are well formed
        assert len(server_matches) == len(client_matches)
        assert len(server_tmatches) == len(client_tmatches)
        assert len(server_idxs) == len(client_idxs)
        assert len(server_matches) == len(server_tmatches)
        assert len(server_matches) == len(server_idxs)

        # Compute clear values
        ids = []
        for cmatch, smatch, ctmatch, stmatch, cidx, sidx in zip(
                client_matches, server_matches, client_tmatches, server_tmatches, client_idxs, server_idxs):
            # In the following three lines the linkage result is "decrypted", i.e., reconstructed from the shares.
            # It consists of match-bit, tentative match bit, and best matching server-patient ID.
            # This is the critical part of the code.
            match = cmatch ^ smatch
            tmatch = ctmatch ^ stmatch
            bestind = cidx ^ sidx

            if match or tmatch:
                # Decrypt ID with server key and encrypt with client key prepend tmatch bit
                # A match set implies a tmatch set.
                # If it's a match or a tentative match, we prefix the decrypted
                # id from the server with the tentative match bit and encrypt
                # everything with the client's key.
                client_linkage_id = client.encrypt(
                    bytes([tmatch]) +  # Convert to bytes of length 1
                    server.decrypt(
                        server_ids[bestind],
                        False), True)
                # Ensure ID is string
                if isinstance(client_linkage_id, bytes):
                    client_linkage_id = client_linkage_id.decode('utf-8')

                # Output is differently formated,
                # based on one or multiple results.
                # (1:n linkage or n:m linkage)
                single = len(client_matches) == 1
                if single:
                    return json.dumps(
                        {'result': {'linkageId': client_linkage_id}})
                else:
                    ids.append({'linkageId': client_linkage_id})

            # If no match...
            else:
                single = len(client_matches) == 1
                if single:
                    # ...we generate a random fresh ID for the client
                    return json.dumps(
                        {'result': {'linkageId': client.generate_id()}})
                else:
                    ids.append({'linkageId': client.generate_id()})

        for i in range(len(ids)):
            if isinstance(ids[i]['linkageId'], bytes):
                ids[i]['linkageId'] = ids[i]['linkageId'].decode('utf-8')

        return json.dumps({'result': ids})
