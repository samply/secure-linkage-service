from linkserv.linkresult import LinkageResult
from linkserv.secureepilinker import SecureEpilinker


class LinkageServer:
    def __init__(self):
        # both are maps/dictionaries
        self.epilinker = {}
        self.computations = {}

    def add_epilinker(self, epilinker_id, key, padding):
        self.epilinker[epilinker_id] = SecureEpilinker(
            epilinker_id, key, padding)

    def generate_ids(self, remote_id, count):
        if remote_id not in self.epilinker:
            raise KeyError('Invalid Remote Id')

        # Convert bytes to strings for easier JSON serialization
        return list(map(bytes.decode,
                        self.epilinker[remote_id].generate_id(count)))

    def get_epilinker(self, epilinker_id):
        if epilinker_id in self.epilinker:
            return self.epilinker[epilinker_id]
        raise KeyError('Invalid Remote Id')

    def set_share(self, local_id, remote_id, data):
        if data['role'] == 'client':
            client_id = local_id
            server_id = remote_id
        elif data['role'] == 'server':
            client_id = remote_id
            server_id = local_id
        else:
            raise KeyError('Invalid Role')

        # multiple computations can be processed in parallel but only one per server-client pair. Very inefficiently handled here.
        # This is a "bookkeeping" section, where we check that every piece of
        # data gets sent to the right place.
        if client_id in self.computations:
            if server_id in self.computations[client_id]:
                self.computations[client_id][server_id].set_share(data)
            else:
                self.computations[client_id][server_id] = LinkageResult(
                    self.get_epilinker(client_id), self.get_epilinker(server_id))
                self.computations[client_id][server_id].set_share(data)
        else:
            self.computations[client_id] = {}
            self.computations[client_id][server_id] = LinkageResult(
                self.get_epilinker(client_id), self.get_epilinker(server_id))
            self.computations[client_id][server_id].set_share(data)

    def is_result_ready(self, client_id, server_id):
        return self.computations[client_id][server_id].is_ready()

    def get_linkage_result(self, client_id, server_id):
        linkage_result = self.computations[client_id][server_id]

        # Create client data
        client_data = {
            'result': [
                {
                    'match': match,
                    'tentativeMatch': tmatch,
                    'bestIndex': idx
                }
                for match, tmatch, idx in zip(
                    linkage_result.client_matches,
                    linkage_result.client_tmatches,
                    linkage_result.client_idxs
                )
            ]
        }

        # Create server data
        server_data = {
            'result': [
                {
                    'match': match,
                    'tentativeMatch': tmatch,
                    'bestIndex': idx
                }
                for match, tmatch, idx in zip(
                    linkage_result.server_matches,
                    linkage_result.server_tmatches,
                    linkage_result.server_idxs
                )
            ],
            'ids': linkage_result.server_ids
        }

        # Use the static method directly
        result = LinkageResult.get_linkage_result(
            client_data,
            server_data,
            self.epilinker[client_id],
            self.epilinker[server_id]
        )

        # Reset the state after computation
        self.reset_result(client_id, server_id)

        return result

    def reset_result(self, client_id, server_id):
        self.computations[client_id][server_id].reset()
