use quote::quote;
use std::collections::HashMap;
use std::iter;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprCall, ExprParen, ExprPath, ExprReference, Ident, Macro, Path, PathSegment};

struct MigrationCtx(HashMap<&'static str, &'static str>);

impl visit_mut::VisitMut for MigrationCtx {
	fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
		if let Ok(mut expr) = i.parse_body_with(Punctuated::<Expr, Comma>::parse_terminated) {
			for expr in expr.iter_mut() {
				self.visit_expr_mut(expr);
			}

			*i = Macro {
				path: i.path.clone(),
				bang_token: i.bang_token,
				delimiter: i.delimiter.clone(),
				tokens: quote! { #expr },
			};
		}
	}

	fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
		visit_mut::visit_expr_mut(self, i);

		match i {
			Expr::MethodCall(call) if call.method == "faer_add" => {
				*i = Expr::Binary(syn::ExprBinary {
					attrs: vec![],
					left: call.receiver.clone(),
					op: syn::BinOp::Add(Default::default()),
					right: Box::new(call.args[0].clone()),
				});

				*i = Expr::Paren(ExprParen {
					attrs: vec![],
					paren_token: Default::default(),
					expr: Box::new(i.clone()),
				});
			},
			Expr::MethodCall(call) if call.method == "faer_sub" => {
				*i = Expr::Binary(syn::ExprBinary {
					attrs: vec![],
					left: call.receiver.clone(),
					op: syn::BinOp::Sub(Default::default()),
					right: Box::new(call.args[0].clone()),
				});

				*i = Expr::Paren(ExprParen {
					attrs: vec![],
					paren_token: Default::default(),
					expr: Box::new(i.clone()),
				});
			},
			Expr::MethodCall(call) if call.method == "faer_mul" => {
				*i = Expr::Binary(syn::ExprBinary {
					attrs: vec![],
					left: call.receiver.clone(),
					op: syn::BinOp::Mul(Default::default()),
					right: Box::new(call.args[0].clone()),
				});

				*i = Expr::Paren(ExprParen {
					attrs: vec![],
					paren_token: Default::default(),
					expr: Box::new(i.clone()),
				});
			},
			Expr::MethodCall(call) if call.method == "faer_div" => {
				*i = Expr::Binary(syn::ExprBinary {
					attrs: vec![],
					left: call.receiver.clone(),
					op: syn::BinOp::Div(Default::default()),
					right: Box::new(call.args[0].clone()),
				});

				*i = Expr::Paren(ExprParen {
					attrs: vec![],
					paren_token: Default::default(),
					expr: Box::new(i.clone()),
				});
			},
			Expr::MethodCall(call) if call.method == "faer_neg" => {
				*i = Expr::Unary(syn::ExprUnary {
					attrs: vec![],
					op: syn::UnOp::Neg(Default::default()),
					expr: call.receiver.clone(),
				});

				*i = Expr::Paren(ExprParen {
					attrs: vec![],
					paren_token: Default::default(),
					expr: Box::new(i.clone()),
				});
			},

			Expr::MethodCall(call) if call.method.to_string().starts_with("faer_") => {
				if let Some(new_method) = self.0.get(&*call.method.to_string()).map(|x| &**x) {
					*i = math_expr(
						&Ident::new(new_method, call.method.span()),
						std::iter::once(&*call.receiver).chain(call.args.iter()),
					)
				}
			},

			_ => {},
		}
	}
}

struct MathCtx;

fn ident_expr(ident: &syn::Ident) -> Expr {
	Expr::Path(ExprPath {
		attrs: vec![],
		qself: None,
		path: Path {
			leading_colon: None,
			segments: Punctuated::from_iter(iter::once(PathSegment {
				ident: ident.clone(),
				arguments: syn::PathArguments::None,
			})),
		},
	})
}

impl visit_mut::VisitMut for MathCtx {
	fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
		if let Ok(mut expr) = i.parse_body_with(Punctuated::<Expr, Comma>::parse_terminated) {
			for expr in expr.iter_mut() {
				self.visit_expr_mut(expr);
			}

			*i = Macro {
				path: i.path.clone(),
				bang_token: i.bang_token,
				delimiter: i.delimiter.clone(),
				tokens: quote! { #expr },
			};
		}
	}

	fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
		visit_mut::visit_expr_mut(self, i);

		match i {
			Expr::Unary(unary) => match unary.op {
				syn::UnOp::Neg(minus) => {
					*i = Expr::Call(ExprCall {
						attrs: vec![],
						func: Box::new(ident_expr(&Ident::new("neg", minus.span))),
						paren_token: Default::default(),
						args: std::iter::once((*unary.expr).clone())
							.map(|e| {
								Expr::Reference(ExprReference {
									attrs: vec![],
									and_token: Default::default(),
									mutability: None,
									expr: Box::new(e),
								})
							})
							.collect(),
					})
				},
				_ => {},
			},
			Expr::Binary(binop) => {
				let func = match binop.op {
					syn::BinOp::Add(plus) => Some(Ident::new("add", plus.span)),
					syn::BinOp::Sub(minus) => Some(Ident::new("sub", minus.span)),
					syn::BinOp::Mul(star) => Some(Ident::new("mul", star.span)),
					syn::BinOp::Div(star) => Some(Ident::new("div", star.span)),
					_ => None,
				};

				if let Some(func) = func {
					*i = Expr::Call(ExprCall {
						attrs: vec![],
						func: Box::new(ident_expr(&func)),
						paren_token: Default::default(),
						args: [(*binop.left).clone(), (*binop.right).clone()]
							.into_iter()
							.map(|e| {
								Expr::Reference(ExprReference {
									attrs: vec![],
									and_token: Default::default(),
									mutability: None,
									expr: Box::new(e),
								})
							})
							.collect(),
					})
				}
			},

			Expr::Call(call) => match &*call.func {
				Expr::Path(e) if e.path.get_ident().is_some() => {
					let name = &*e.path.get_ident().unwrap().to_string();

					if matches!(
						name,
						"sqrt"
							| "from_real" | "copy"
							| "max" | "min" | "conj"
							| "absmax" | "abs2" | "abs1"
							| "abs" | "add" | "sub"
							| "div" | "mul" | "mul_real"
							| "mul_pow2" | "hypot"
							| "neg" | "recip" | "real"
							| "imag" | "is_nan" | "is_finite"
							| "is_zero" | "lt_zero"
							| "gt_zero" | "le_zero"
							| "ge_zero"
					) {
						call.args.iter_mut().for_each(|x| {
							*x = Expr::Reference(ExprReference {
								attrs: vec![],
								and_token: Default::default(),
								mutability: None,
								expr: Box::new(x.clone()),
							})
						})
					}
				},
				_ => {},
			},
			_ => {},
		}
	}
}

fn math_expr<'a>(method: &Ident, args: impl Iterator<Item = &'a Expr>) -> Expr {
	Expr::Call(ExprCall {
		attrs: vec![],

		paren_token: Default::default(),
		args: args.cloned().collect(),
		func: Box::new(ident_expr(method)),
	})
}

#[proc_macro_attribute]

pub fn math(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let Ok(mut item) = syn::parse::<syn::ItemFn>(item.clone()) else {
		return item;
	};

	let mut rust_ctx = MathCtx;

	rust_ctx.visit_item_fn_mut(&mut item);

	let item = quote! { #item };

	item.into()
}

#[proc_macro_attribute]

pub fn migrate(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let Ok(mut item) = syn::parse::<syn::ItemFn>(item.clone()) else {
		return item;
	};

	let mut rust_ctx = MigrationCtx(
		[
			//
			("faer_add", "add"),
			("faer_sub", "sub"),
			("faer_mul", "mul"),
			("faer_div", "div"),
			("faer_neg", "neg"),
			("faer_inv", "recip"),
			("faer_abs", "abs"),
			("faer_abs2", "abs2"),
			("faer_sqrt", "sqrt"),
			("faer_conj", "conj"),
			("faer_real", "real"),
			("faer_scale_real", "mul_real"),
			("faer_scale_power_of_two", "mul_pow2"),
		]
		.into_iter()
		.collect(),
	);

	rust_ctx.visit_item_fn_mut(&mut item);

	let mut rust_ctx = MathCtx;

	rust_ctx.visit_item_fn_mut(&mut item);

	let item = quote! { #item };

	item.into()
}
