use serde::{Deserialize, Serialize};
use serde_json::json;
use snarkvm_console_network::Network;
use snarkvm_console_types_address::{Address, SerializeStruct};
use snarkvm_ledger_coinbase::{EpochChallenge, ProverSolution};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub enum PoolMessage<N: Network> {
    #[serde(rename = "auth_request")]
    AuthRequest(AuthRequest),
    #[serde(rename = "auth_response")]
    AuthResponse(AuthResponse<N>),
    #[serde(rename = "task")]
    NewTask(NewTask<N>),
    #[serde(rename = "solution")]
    NewSolution(NewSolution<N>),
}

impl<N: Network> PoolMessage<N> {
    pub fn auth_request(self) -> Option<AuthRequest> {
        match self {
            Self::AuthRequest(request) => Some(request),
            _ => None,
        }
    }

    pub fn auth_response(self) -> Option<AuthResponse<N>> {
        match self {
            Self::AuthResponse(response) => Some(response),
            _ => None,
        }
    }

    pub fn new_task(self) -> Option<NewTask<N>> {
        match self {
            Self::NewTask(new_task) => Some(new_task),
            _ => None,
        }
    }

    pub fn new_solution(self) -> Option<NewSolution<N>> {
        match self {
            Self::NewSolution(new_solution) => Some(new_solution),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    // miner account or aleo address
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct AuthResponse<N: Network> {
    // Indicate the `AuthRequest` is success or not.
    pub result: bool,
    // Pool address.
    pub address: Address<N>,
    // Extra message
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct NewTask<N: Network> {
    pub epoch_challenge: EpochChallenge<N>,
    pub difficulty: u64,
}

#[derive(Debug)]
pub struct NewSolution<N: Network> {
    epoch_number: u32,
    solution: ProverSolution<N>,
}

impl<N: Network> serde::ser::Serialize for NewTask<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut m_ctx = serializer.serialize_struct("NewTask", false as usize + 1 + 1)?;
        // epoch challenge
        let mut inner = serde_json::Map::new();
        inner.insert(
            "epoch_number".to_owned(),
            self.epoch_challenge.epoch_number().into(),
        );
        inner.insert(
            "epoch_block_hash".to_owned(),
            json!(self.epoch_challenge.epoch_block_hash()),
        );
        inner.insert("degree".to_owned(), self.epoch_challenge.degree().into());
        m_ctx.serialize_field("epoch_challenge", &inner)?;

        m_ctx.serialize_field("difficulty", &self.difficulty)?;

        m_ctx.end()
    }
}

impl<'de, N: Network> serde::de::Deserialize<'de> for NewTask<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut new_work = serde_json::Value::deserialize(deserializer)?;
        // epoch challenge
        let inner: serde_json::Map<_, _> =
            serde_json::from_value(new_work["epoch_challenge"].take())
                .map_err(serde::de::Error::custom)?;
        let epoch_number = inner
            .get("epoch_number")
            .ok_or(serde::de::Error::custom("missing epoch_number"))?;
        let epoch_number: u32 =
            serde_json::from_value(epoch_number.clone()).map_err(serde::de::Error::custom)?;

        let epoch_block_hash = inner
            .get("epoch_block_hash")
            .ok_or(serde::de::Error::custom("missing epoch_block_hash"))?;
        let epoch_block_hash: N::BlockHash =
            serde_json::from_value(epoch_block_hash.clone()).map_err(serde::de::Error::custom)?;

        let degree = inner
            .get("degree")
            .ok_or(serde::de::Error::custom("missing degree"))?;
        let degree: u32 =
            serde_json::from_value(degree.clone()).map_err(serde::de::Error::custom)?;

        let difficulty: u64 = serde_json::from_value(new_work["difficulty"].take())
            .map_err(serde::de::Error::custom)?;

        Ok(Self {
            epoch_challenge: EpochChallenge::new(epoch_number, epoch_block_hash, degree)
                .map_err(serde::de::Error::custom)?,
            difficulty,
        })
    }
}

impl<N: Network> Serialize for NewSolution<N> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut m_ctx = serializer.serialize_struct("NewSolution", false as usize + 1 + 1)?;
        m_ctx.serialize_field("solution", &self.solution)?;
        m_ctx.serialize_field("epoch_number", &self.epoch_number)?;
        m_ctx.end()
    }
}

impl<'de, N: Network> Deserialize<'de> for NewSolution<N> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut m_result = serde_json::Value::deserialize(deserializer)?;
        let solution: ProverSolution<N> = serde_json::from_value(m_result["solution"].take())
            .map_err(serde::de::Error::custom)?;
        let epoch_number: u32 = serde_json::from_value(m_result["epoch_number"].take())
            .map_err(serde::de::Error::custom)?;

        Ok(Self {
            solution,
            epoch_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;
    use snarkvm_console_network::Testnet3;
    use snarkvm_console_types_address::FromStr;
    use snarkvm_ledger_coinbase::PartialSolution;
    use snarkvm_utilities::TestRng;

    #[test]
    fn test_serialize_auth_request() {
        let message = PoolMessage::<Testnet3>::AuthRequest(AuthRequest {
            username: "test_username".to_string(),
        });
        println!("{}", json!(message));
    }

    #[test]
    fn test_serialize_auth_response() {
        let address: Address<Testnet3> =
            Address::from_str("aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px")
                .unwrap();
        let message = PoolMessage::<Testnet3>::AuthResponse(AuthResponse {
            result: true,
            address,
            message: None,
        });
        println!("{}", json!(message))
    }
    #[test]
    fn test_serialize_new_task() {
        let epoch_challenge = EpochChallenge::new(0, Default::default(), (1 << 13) - 1).unwrap();
        let message = PoolMessage::<Testnet3>::NewTask(NewTask {
            epoch_challenge,
            difficulty: 0,
        });
        let serialized = json!(message);
        println!("{}", serialized);
        let message: PoolMessage<Testnet3> = serde_json::from_str(&serialized.to_string()).unwrap();
        assert!(matches!(message, PoolMessage::NewTask(..)));
    }

    #[test]
    fn test_serialize_new_solution() {
        use snarkvm_algorithms::polycommit::kzg10::KZGCommitment;
        use snarkvm_algorithms::polycommit::kzg10::KZGProof;
        use snarkvm_console_types_address::Rng;
        use snarkvm_utilities::Uniform;
        let mut rng = TestRng::default();
        let address: Address<Testnet3> =
            Address::from_str("aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px")
                .unwrap();
        let partial_solution =
            PartialSolution::new(address, u64::rand(&mut rng), KZGCommitment(rng.gen()));
        let solution = ProverSolution::new(
            partial_solution,
            KZGProof {
                w: rng.gen(),
                random_v: None,
            },
        );
        let message = PoolMessage::<Testnet3>::NewSolution(NewSolution {
            solution,
            epoch_number: 0,
        });
        let serialized = json!(message);
        println!("{}", serialized);
        let message: PoolMessage<Testnet3> = serde_json::from_str(&serialized.to_string()).unwrap();
        assert!(matches!(message, PoolMessage::NewSolution(..)))
    }
}
