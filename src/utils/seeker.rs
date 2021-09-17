use crate::utils::misc::*;
use duplicate::*;
use std::default::Default;
//for values to seek over
pub trait Quantify {
    type Quantifier: PartialOrd;

    fn quantify(&self) -> Self::Quantifier;
}

//for seeker
pub trait SeekerTypes {
    type Source: Quantify; //in case of meta seekers this is the leader
    type Output;
}

pub trait Seek: SeekerTypes {
    fn seek(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
    fn jump(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

pub trait Exhibit: SeekerTypes {
    fn exhibit(& self, t: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

//for collection of seekable values
pub trait Seekable<'a> {
    type Seeker: Seek;
    
    fn seeker(&'a self) -> Self::Seeker;
}

pub trait SeekExtensions
{
    type Item: Quantify;

    fn quantified_insert(&mut self, item: Self::Item) -> usize;
}
//
//
//
//
//
#[derive(Clone, Copy, Default, Debug)]
pub struct Epoch<Value> {
    pub offset: f32,
    pub val: Value,
}

impl<Value> Quantify for Epoch<Value>
{
    type Quantifier = f32;
    fn quantify(&self) -> Self::Quantifier {
        self.offset
    }
}

impl<Value> From<(f32, Value)> for Epoch<Value>
where
    Value: Copy,
{
    fn from(tup: (f32, Value)) -> Epoch<Value> {
        Epoch::<Value> {
            offset: tup.0,
            val: tup.1,
        }
    }
}

impl<'a, T> SeekerTypes for Seeker<&'a [Epoch<T>], usize> 
where
    T: Copy,
{
    type Source = Epoch<T>;
    type Output = T;
}

impl<'a, T, U, V> SeekerTypes for Seeker<&'a [Epoch<T>], Seeker<U, V>>
where
    T: Seekable<'a, Seeker = Seeker<U, V>>
{
    type Source = Epoch<T>;
    type Output = Seeker<U, V>;
}
//
//
//
//
//
pub struct Seeker<Data, Meta>
{
    pub data: Data, //unchanging
    pub meta: Meta, //changing
}

pub type Output<'a, T> = <T as SeekerTypes>::Output;
pub type Quantifier<'a, T> = <<T as SeekerTypes>::Source as Quantify>::Quantifier;


    
impl<'a, T> Seeker<&'a [T], usize>
where
    T: Quantify
{
    pub fn current(&self) -> Result<&T, &T> {
        if self.meta < self.data.len() {
            Ok(&self.data[self.meta])
        }
        else {
            Err(&self.data[FromEnd(0)])
        }
    }

    pub fn previous(&self) -> Option<&T> {
        if 1 < self.data.len() && 0 < self.meta {
            Some(&self.data[self.meta - 1])
        }
        else {
            None
        }
    }

    pub fn next(&self) -> Option<&T> {
        if 1 < self.data.len() && self.meta - 1 < self.data.len() {
            Some(&self.data[self.meta + 1])
        }
        else {
            None
        }
    }
}

impl<'a, T> Seek for Seeker<&'a [T], usize>
where
    T: Quantify,
    Self: Exhibit<Source = T>
{
    fn seek(&mut self, offset: Quantifier<'a, Self>) -> Output<'a, Self>
    {
        while self.meta < self.data.len() {
            if offset < self.data[self.meta].quantify() {
                break;
            }
            self.meta += 1;
        }
        self.exhibit(offset)
    }

    fn jump(&mut self, offset: Quantifier<'a, Self>) -> Output<'a, Self> {
        self.meta = match self
            .data
            .binary_search_by(|elem| elem.quantify().partial_cmp(&offset).unwrap())
        {
            Ok(index) => index,
            Err(index) => index 
        };
        self.exhibit(offset)
    }
}

impl<'a, T> Seekable<'a> for &'a [T]
where
    T: Quantify,
    Seeker<&'a [T], usize>: Exhibit<Source = T>
{
    type Seeker = Seeker<&'a [T], usize>;
    
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            meta: 0,
            data: self
        }
    }
}

impl<'a, T, U, V> Seekable<'a> for &'a [Epoch<T>]
where
    T: Seekable<'a, Seeker = Seeker<U, V>>,
    Seeker<(), (Seeker<&'a [Epoch<T>], usize>, Seeker<U, V>)>: Seek
{
    type Seeker = Seeker<(), (Seeker<&'a [Epoch<T>], usize>, Seeker<U, V>)>;

    fn seeker(&'a self) -> Self::Seeker {
        Seeker {
            data: (),
            meta: (self.seeker(), self[0].val.seeker())
        }
    }
}

/*#[duplicate(
    VecT        D;
    [Vec<T>]    [];
    [TVec<T>]   [Default]
)]
impl<T> SeekExtensions for VecT
where
    T: Quantify + Copy + D,
{
    type Item = T;
    fn quantified_insert(&mut self, item: T) -> usize {
        let index = match self
            .as_slice()
            .binary_search_by(|a| a.quantify().partial_cmp(&item.quantify()).unwrap()) {
                Ok(index) | Err(index) => index,
        };
        self.insert(index, item);
        index
    }
}*/
