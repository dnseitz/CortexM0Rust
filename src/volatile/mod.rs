
mod tests;

use core::ops::*;
use core::intrinsics::{volatile_load, volatile_store};

#[derive(Copy, Clone)]
struct Volatile<T: Copy>(RawVolatile<T>);

#[derive(Copy, Clone)]
struct RawVolatile<T: Copy>(*const T);

impl<T: Copy> Volatile<T> {
  unsafe fn new(ptr: *const T) -> Self {
    Volatile(RawVolatile::new(ptr))
  }
}

impl<T: Copy> Deref for Volatile<T> {
  type Target = RawVolatile<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: Copy> DerefMut for Volatile<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T: Copy> RawVolatile<T> {
  fn new(ptr: *const T) -> Self {
    RawVolatile(ptr)
  }

  fn store(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, rhs);
    }
  }

  fn load(&self) -> T {
    unsafe {
      volatile_load(self.0 as *const T)
    }
  }
}

impl<T: Add<Output=T> + Copy> Add<T> for RawVolatile<T> {
  type Output = T;

  fn add(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) + rhs
    }
  }
}

impl<T: Add<Output=T> + Copy> AddAssign<T> for RawVolatile<T> {
  fn add_assign(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, volatile_load(self.0) + rhs);
    }
  }
}

impl<T: Sub<Output=T> + Copy> Sub<T> for RawVolatile<T> {
  type Output = T;

  fn sub(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) - rhs
    }
  }
}

impl<T: Sub<Output=T> + Copy> SubAssign<T> for RawVolatile<T> {
  fn sub_assign(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, volatile_load(self.0) - rhs);
    }
  }
}

impl<T: Mul<Output=T> + Copy> Mul<T> for RawVolatile<T> {
  type Output = T;

  fn mul(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) * rhs
    }
  }
}

impl<T: Mul<Output=T> + Copy> MulAssign<T> for RawVolatile<T> {
  fn mul_assign(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, volatile_load(self.0) * rhs);
    }
  }
}

impl<T: Div<Output=T> + Copy> Div<T> for RawVolatile<T> {
  type Output = T;
  
  fn div(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) / rhs
    }
  }
}

impl<T: Div<Output=T> + Copy> DivAssign<T> for RawVolatile<T> {
  fn div_assign(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, volatile_load(self.0) / rhs);
    }
  }
}

/*** Bitwise Operators ***/
impl<T: BitAnd<Output=T> + Copy> BitAnd<T> for RawVolatile<T> {
  type Output = T;

  fn bitand(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) & rhs
    }
  }
}

impl<T: BitAnd<Output=T> + Copy> BitAndAssign<T> for RawVolatile<T> {
  fn bitand_assign(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, volatile_load(self.0) & rhs);
    }
  }
}

impl<T: BitOr<Output=T> + Copy> BitOr<T> for RawVolatile<T> {
  type Output = T;

  fn bitor(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) | rhs
    }
  }
}

impl<T: BitOr<Output=T> + Copy> BitOrAssign<T> for RawVolatile<T> {
  fn bitor_assign(&mut self, rhs: T) {
    unsafe {
      volatile_store(self.0 as *mut T, volatile_load(self.0) | rhs);
    }
  }
}

impl<T: BitXor<Output=T> + Copy> BitXor<T> for RawVolatile<T> {
  type Output = T;

  fn bitxor(self, rhs: T) -> Self::Output {
    unsafe {
      volatile_load(self.0) ^ rhs
    }
  }
}
