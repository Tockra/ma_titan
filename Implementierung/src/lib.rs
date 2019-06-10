

pub trait PredecessorList<T> {
    fn insert(&self,T);
    fn delete(&self,T);
    fn predecessor(&self,T) -> Option<T>;
    fn sucessor(&self,T) -> Option<T>;
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>; 
}


struct STree<T: Num>