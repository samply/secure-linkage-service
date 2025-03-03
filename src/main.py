from flask import Flask
from flask import jsonify
from flask import request
from flask import make_response
from linkserv.linkageserver import LinkageServer
from Crypto import Random
import simplejson as json
import time
import os

app = Flask(__name__)

LinServ = LinkageServer()
# Manually register the sites. First argument is ID, second is a symmetric key, third is the number of trailing zeros for ID validation (more on that at the id generation code)
# Each side has a specific number of zeroes. That's how it is identified.
# There are beter ways to do this.

# The third parameter should be eventually removed, and the ID validation
# process should be performed within the encryption process.

LinServ.add_epilinker("tuda", Random.new().read(32), 15)
LinServ.add_epilinker("dkfz", Random.new().read(32), 13)
LinServ.add_epilinker("charite", Random.new().read(32), 17)

# This is arbitrary
timeouttime = 20

# Just a basic health check


@app.route('/', methods=['GET'])
def running():
    return 'Server is running!'

# Connectivity check


@app.route('/testConnection/<string:remote_id>', methods=['GET'])
def check_connection(remote_id):
    print("Got test request from", remote_id)
    try:
        LinServ.get_epilinker(remote_id)
        return make_response('', 204)
    except KeyError as e:
        return make_response('Host not configured', 403)
    # Look for a better error code.
    # Maybe 500?


# Generate new ids for the server-site
# We should add error handling
# What if
# * the remote ID is not registered?
# * the count is < 1?
@app.route('/freshIDs/<string:remote_id>', methods=['GET'])
def get_ids(remote_id):
    count = int(request.args.get('count', 1))
    ids = LinServ.generate_ids(remote_id, count)
    return jsonify({'linkageIds': ids})

# Weird duplicate, please ignore
# We can probably delete this endpoint.
# (Look it up in the documentation.)


@app.route('/testConnection/<string:remote_id>')
def test_connection(remote_id):
    resp = make_response('Connection correctly established', 200)
    return resp

# This is the workflow documented in the diagram, and in section 2.4.2.
#
# This call is asynchrounous for the server; it only sets its results. It
# is synchrounous for the client. It sets ts results, waits unil both
# shares are set and then returns the linkage result (i.e. a linkage id)


@app.route('/linkageResult/<string:local_id>/<string:remote_id>',
           methods=['POST'])
def linkageResult(local_id, remote_id):
    length = request.content_length
    if length != 0:
        try:
            data = request.get_json()
            if data is not None:
                LinServ.set_share(local_id, remote_id, data)
                if data['role'] == 'client':
                    print("Client")
                    timeout_counter = 0
                    timeout = False
                    while not LinServ.is_result_ready(local_id, remote_id):
                        print("Waiting for Server")
                        time.sleep(1)
                        timeout_counter += 1
                        if timeout_counter >= timeouttime:
                            timeout = True
                            break
                    if not timeout:
                        result = LinServ.get_linkage_result(
                            local_id, remote_id)
                        LinServ.clear_result(local_id, remote_id)
                        return result
                    LinServ.clear_result(local_id, remote_id)
                    return make_response('No result from server', 408)
                # Check whether 408 is actually correct

                return 'Received server result'
            return make_response('Invalid JSON', 400)
        except Exception as e:
            print(type(e), e)
            raise e
            return make_response('Invalid JSON', 400)
    else:
        return make_response('No result sent', 411)
    # Check whether 411 is correct?


if __name__ == "__main__":
    no_tls = os.getenv("LS_NO_TLS")
    if no_tls is None:
        context = ('server.crt', 'server.key')
        app.run(
            host='0.0.0.0',
            port=5000,
            ssl_context=context,
            threaded=True,
            debug=False)
    else:
        app.run(host='0.0.0.0', port=5000, threaded=True, debug=False)
