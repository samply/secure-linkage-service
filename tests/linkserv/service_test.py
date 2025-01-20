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
