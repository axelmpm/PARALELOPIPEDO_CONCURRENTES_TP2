
pub struct PendingStorage{
    pending_confirmation: HashSet<String>,
}

impl PendingStorage{
    
    pub fn new(){
        return PendingStore{};
    }

    pub fn store(&self, value: String){
        pending_confirmation.insert(value);
    }

    pub fn store