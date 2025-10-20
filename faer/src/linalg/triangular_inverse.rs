use crate::internal_prelude::*;
use crate::utils::thread::join_raw;
use linalg::matmul::triangular::BlockStructure;

fn invert_lower_triangular_impl_small<'N, T: ComplexField>(mut dst: MatMut<'_, T, Dim<'N>, Dim<'N>>, src: MatRef<'_, T, Dim<'N>, Dim<'N>>) {
	let N = dst.nrows();

	match *N {
		0 => {},
		1 => {
			let i0 = N.check(0);

			*dst.rb_mut().at_mut(i0, i0) = src[(i0, i0)].recip();
		},
		2 => {
			let i0 = N.check(0);

			let i1 = N.check(1);

			let dst00 = src[(i0, i0)].recip();

			let dst11 = src[(i1, i1)].recip();

			let dst10 = -&dst11 * &src[(i1, i0)] * &dst00;

			*dst.rb_mut().at_mut(i0, i0) = dst00;

			*dst.rb_mut().at_mut(i1, i1) = dst11;

			*dst.rb_mut().at_mut(i1, i0) = dst10;
		},
		_ => unreachable!(),
	}
}

fn invert_unit_lower_triangular_impl_small<'N, T: ComplexField>(mut dst: MatMut<'_, T, Dim<'N>, Dim<'N>>, src: MatRef<'_, T, Dim<'N>, Dim<'N>>) {
	let N = dst.nrows();

	match *N {
		0 | 1 => {},
		2 => {
			let i0 = N.check(0);

			let i1 = N.check(1);

			*dst.rb_mut().at_mut(i1, i0) = -&src[(i1, i0)];
		},
		_ => unreachable!(),
	}
}

fn invert_lower_triangular_impl<'N, T: ComplexField>(dst: MatMut<'_, T, Dim<'N>, Dim<'N>>, src: MatRef<'_, T, Dim<'N>, Dim<'N>>, par: Par) {
	let N = dst.ncols();

	if *N <= 2 {
		invert_lower_triangular_impl_small(dst, src);

		return;
	}

	make_guard!(HEAD);

	make_guard!(TAIL);

	let mid = N.partition(N.checked_idx_inc(*N / 2), HEAD, TAIL);

	let (mut dst_tl, _, mut dst_bl, mut dst_br) = { dst.split_with_mut(mid, mid) };

	let (src_tl, _, src_bl, src_br) = { src.split_with(mid, mid) };

	join_raw(
		|par| invert_lower_triangular_impl(dst_tl.rb_mut(), src_tl, par),
		|par| invert_lower_triangular_impl(dst_br.rb_mut(), src_br, par),
		par,
	);

	linalg::matmul::triangular::matmul(
		dst_bl.rb_mut(),
		BlockStructure::Rectangular,
		Accum::Replace,
		src_bl,
		BlockStructure::Rectangular,
		dst_tl.rb(),
		BlockStructure::TriangularLower,
		-one::<T>(),
		par,
	);

	linalg::triangular_solve::solve_lower_triangular_in_place(src_br, dst_bl, par);
}

fn invert_unit_lower_triangular_impl<'N, T: ComplexField>(dst: MatMut<'_, T, Dim<'N>, Dim<'N>>, src: MatRef<'_, T, Dim<'N>, Dim<'N>>, par: Par) {
	let N = dst.ncols();

	if *N <= 2 {
		invert_unit_lower_triangular_impl_small(dst, src);

		return;
	}

	make_guard!(HEAD);

	make_guard!(TAIL);

	let mid = N.partition(N.checked_idx_inc(*N / 2), HEAD, TAIL);

	let (mut dst_tl, _, mut dst_bl, mut dst_br) = { dst.split_with_mut(mid, mid) };

	let (src_tl, _, src_bl, src_br) = { src.split_with(mid, mid) };

	join_raw(
		|par| invert_unit_lower_triangular_impl(dst_tl.rb_mut(), src_tl, par),
		|par| invert_unit_lower_triangular_impl(dst_br.rb_mut(), src_br, par),
		par,
	);

	linalg::matmul::triangular::matmul(
		dst_bl.rb_mut(),
		BlockStructure::Rectangular,
		Accum::Replace,
		src_bl,
		BlockStructure::Rectangular,
		dst_tl.rb(),
		BlockStructure::UnitTriangularLower,
		-one::<T>(),
		par,
	);

	linalg::triangular_solve::solve_unit_lower_triangular_in_place(src_br, dst_bl, par);
}

/// computes the inverse of the lower triangular matrix `src` (with implicit unit
/// diagonal) and stores the strictly lower triangular part of the result to `dst`.
///
/// # panics
///
/// panics if `src` and `dst` have mismatching dimensions, or if they are not square.
#[track_caller]

pub fn invert_unit_lower_triangular<T: ComplexField>(dst: MatMut<'_, T>, src: MatRef<'_, T>, par: Par) {
	Assert!(all(dst.nrows() == src.nrows(), dst.ncols() == src.ncols(), dst.nrows() == dst.ncols()));

	with_dim!(N, dst.nrows().unbound());

	invert_unit_lower_triangular_impl(dst.as_shape_mut(N, N).as_dyn_stride_mut(), src.as_shape(N, N).as_dyn_stride(), par)
}

/// computes the inverse of the lower triangular matrix `src` and stores the
/// lower triangular part of the result to `dst`.
///
/// # panics
///
/// panics if `src` and `dst` have mismatching dimensions, or if they are not square.
#[track_caller]

pub fn invert_lower_triangular<T: ComplexField>(dst: MatMut<'_, T>, src: MatRef<'_, T>, par: Par) {
	Assert!(all(dst.nrows() == src.nrows(), dst.ncols() == src.ncols(), dst.nrows() == dst.ncols()));

	with_dim!(N, dst.nrows().unbound());

	invert_lower_triangular_impl(dst.as_shape_mut(N, N).as_dyn_stride_mut(), src.as_shape(N, N).as_dyn_stride(), par)
}

/// computes the inverse of the upper triangular matrix `src` (with implicit unit
/// diagonal) and stores the strictly upper triangular part of the result to `dst`.
///
/// # panics
///
/// panics if `src` and `dst` have mismatching dimensions, or if they are not square.
#[track_caller]

pub fn invert_unit_upper_triangular<T: ComplexField>(dst: MatMut<'_, T>, src: MatRef<'_, T>, par: Par) {
	invert_unit_lower_triangular(dst.reverse_rows_and_cols_mut(), src.reverse_rows_and_cols(), par)
}

/// computes the inverse of the upper triangular matrix `src` and stores the
/// upper triangular part of the result to `dst`.
///
/// # panics
///
/// panics if `src` and `dst` have mismatching dimensions, or if they are not square.
#[track_caller]

pub fn invert_upper_triangular<T: ComplexField>(dst: MatMut<'_, T>, src: MatRef<'_, T>, par: Par) {
	invert_lower_triangular(dst.reverse_rows_and_cols_mut(), src.reverse_rows_and_cols(), par)
}

#[cfg(test)]

mod tests {

	use super::*;
	use crate::{Mat, MatRef, assert};
	use assert_approx_eq::assert_approx_eq;
	use linalg::matmul::triangular;
	use rand::SeedableRng;
	use rand::rngs::StdRng;
	use rand_distr::{Distribution, StandardNormal};

	#[test]

	fn test_invert_lower() {
		let rng = &mut StdRng::seed_from_u64(0);

		(0..32).for_each(|n| {
			let mut a: Mat<f64> = crate::stats::CwiseMatDistribution {
				nrows: n,
				ncols: n,
				dist: StandardNormal,
			}
			.sample(rng);

			a += MatRef::from_repeated_ref(&2.0, n, n);

			let mut inv = Mat::zeros(n, n);

			invert_lower_triangular(inv.as_mut(), a.as_ref(), Par::rayon(0));

			let mut prod = Mat::zeros(n, n);

			triangular::matmul(
				prod.as_mut(),
				BlockStructure::Rectangular,
				Accum::Replace,
				a.as_ref(),
				BlockStructure::TriangularLower,
				inv.as_ref(),
				BlockStructure::TriangularLower,
				1.0,
				Par::rayon(0),
			);

			for i in 0..n {
				for j in 0..n {
					let target = if i == j { 1.0 } else { 0.0 };

					assert_approx_eq!(prod[(i, j)], target, 1e-4);
				}
			}
		});
	}

	#[test]

	fn test_invert_unit_lower() {
		let rng = &mut StdRng::seed_from_u64(0);

		(0..32).for_each(|n| {
			let mut a: Mat<f64> = crate::stats::CwiseMatDistribution {
				nrows: n,
				ncols: n,
				dist: StandardNormal,
			}
			.sample(rng);

			a += MatRef::from_repeated_ref(&2.0, n, n);

			let mut inv = Mat::zeros(n, n);

			invert_unit_lower_triangular(inv.as_mut(), a.as_ref(), Par::rayon(0));

			let mut prod = Mat::zeros(n, n);

			triangular::matmul(
				prod.as_mut(),
				BlockStructure::Rectangular,
				Accum::Replace,
				a.as_ref(),
				BlockStructure::UnitTriangularLower,
				inv.as_ref(),
				BlockStructure::UnitTriangularLower,
				1.0,
				Par::rayon(0),
			);

			for i in 0..n {
				for j in 0..n {
					let target = if i == j { 1.0 } else { 0.0 };

					assert_approx_eq!(prod[(i, j)], target, 1e-4);
				}
			}
		});
	}

	#[test]

	fn test_invert_upper() {
		let rng = &mut StdRng::seed_from_u64(0);

		(0..32).for_each(|n| {
			let mut a: Mat<f64> = crate::stats::CwiseMatDistribution {
				nrows: n,
				ncols: n,
				dist: StandardNormal,
			}
			.sample(rng);

			a += MatRef::from_repeated_ref(&2.0, n, n);

			let mut inv = Mat::zeros(n, n);

			invert_upper_triangular(inv.as_mut(), a.as_ref(), Par::rayon(0));

			let mut prod = Mat::zeros(n, n);

			triangular::matmul(
				prod.as_mut(),
				BlockStructure::Rectangular,
				Accum::Replace,
				a.as_ref(),
				BlockStructure::TriangularUpper,
				inv.as_ref(),
				BlockStructure::TriangularUpper,
				1.0,
				Par::rayon(0),
			);

			for i in 0..n {
				for j in 0..n {
					let target = if i == j { 1.0 } else { 0.0 };

					assert_approx_eq!(prod[(i, j)], target, 1e-4);
				}
			}
		});
	}

	#[test]

	fn test_invert_unit_upper() {
		let rng = &mut StdRng::seed_from_u64(0);

		(0..32).for_each(|n| {
			let mut a: Mat<f64> = crate::stats::CwiseMatDistribution {
				nrows: n,
				ncols: n,
				dist: StandardNormal,
			}
			.sample(rng);

			a += MatRef::from_repeated_ref(&2.0, n, n);

			let mut inv = Mat::zeros(n, n);

			invert_unit_upper_triangular(inv.as_mut(), a.as_ref(), Par::rayon(0));

			let mut prod = Mat::zeros(n, n);

			triangular::matmul(
				prod.as_mut(),
				BlockStructure::Rectangular,
				Accum::Replace,
				a.as_ref(),
				BlockStructure::UnitTriangularUpper,
				inv.as_ref(),
				BlockStructure::UnitTriangularUpper,
				1.0,
				Par::rayon(0),
			);

			for i in 0..n {
				for j in 0..n {
					let target = if i == j { 1.0 } else { 0.0 };

					assert_approx_eq!(prod[(i, j)], target, 1e-4);
				}
			}
		});
	}
}
