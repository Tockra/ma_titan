pub trait PredecessorList<T> {
    fn insert(&self,T);
    fn delete(&self,T);
    fn predecessor(&self,T);
    fn sucessor(&self,T);
    fn minimum(&self);
    fn maximum(&self); 
}

pub mod tests {

}