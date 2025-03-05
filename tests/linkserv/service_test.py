import simplejson as json
import pytest
from main import app, LinServ


@pytest.fixture
def client():
    with app.test_client() as client:
        yield client


def test_health_check(client):
    response = client.get('/')

    assert response.status_code == 200
    assert b"Server is running!" in response.data


@pytest.mark.parametrize("remote_id", ["tuda", "dkfz", "charite"])
def test_check_connection_success(client, mocker, remote_id):
    mocker.patch.object(LinServ, 'get_epilinker', return_value=True)

    response = client.get(f'/testConnection/{remote_id}')

    assert response.status_code == 204
    assert response.data == b''


@pytest.mark.parametrize("remote_id", ["non_existent_host"])
def test_check_connection_failure(client, mocker, remote_id):
    mocker.patch.object(LinServ, 'get_epilinker', side_effect=KeyError)

    response = client.get(f'/testConnection/{remote_id}')

    assert response.status_code == 403
    assert response.data == b'Host not configured'


@pytest.mark.parametrize("remote_id", ["tuda", "dkfz", "charite"])
def test_get_fresh_ids_count_0(client, remote_id):
    response = client.get(f'/freshIDs/{remote_id}?count=0')

    # print(response.data)

    assert response.status_code == 200

    response_json = response.get_json()

    assert response_json is not None
    assert 'linkageIds' in response_json
    assert response_json['linkageIds'] == []


@pytest.mark.parametrize("remote_id", ["tuda", "dkfz", "charite"])
def test_get_fresh_ids_count_1(client, mocker, remote_id):
    expected_ids = ['aGVsbG93b3JsZA==']
    mocker.patch.object(LinServ, 'generate_ids', return_value=expected_ids)

    response = client.get(f'/freshIDs/{remote_id}?count=1')

    # print(response.data)

    assert response.status_code == 200

    response_json = response.get_json()

    assert response_json is not None
    assert 'linkageIds' in response_json
    assert response_json['linkageIds'] == expected_ids


@pytest.mark.parametrize("remote_id", ["tuda", "dkfz", "charite"])
def test_get_fresh_ids_count_3(client, mocker, remote_id):
    expected_ids = ['aGVsbG93b3JsZA==', 'ZGF0YXNjaWVuY2U=', 'cHl0aG9uaXNmdW4=']
    mocker.patch.object(LinServ, 'generate_ids', return_value=expected_ids)

    response = client.get(f'/freshIDs/{remote_id}?count=3')

    # print(response.data)

    assert response.status_code == 200

    response_json = response.get_json()

    assert response_json is not None
    assert 'linkageIds' in response_json
    assert response_json['linkageIds'] == expected_ids


@pytest.mark.parametrize("local_id, remote_id", [("tuda", "dkfz"),
                                                 ("tuda", "charite"),
                                                 ("dkfz", "tuda"),
                                                 ("dkfz", "charite"),
                                                 ("charite", "tuda"),
                                                 ("charite", "dkfz")
                                                 ])
def test_server_linkage_result_role_server(client, local_id, remote_id):

    data = {
        "role": "server",
        "result": [
            {"match": 0, "tentativeMatch": 1, "bestIndex": 2},
            {"match": 1, "tentativeMatch": 0, "bestIndex": 0},
            {"match": 0, "tentativeMatch": 0, "bestIndex": 4}
        ],
        "ids": ["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U=", "cHl0aG9uaXNmdW4=", "cHJpdmFjeXByZXNlcnZpbmc=", "bGlua2FnZXN5c3RlbQ=="]
    }

    response = client.post(
        f'/linkageResult/{local_id}/{remote_id}',
        data=json.dumps(data),
        content_type='application/json'
    )

    assert response.status_code == 200
    assert response.data.decode() == "Received server result"
