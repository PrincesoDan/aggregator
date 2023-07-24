//! This contract demonstrates a sample implementation of the Soroban token
//! interface.
use crate::token::admin::{check_admin, has_administrator, write_administrator};
use crate::token::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::token::balance::{is_authorized, write_authorization};
use crate::token::balance::{read_balance, receive_balance, spend_balance};
use crate::token::event;
use crate::token::metadata::{
    read_decimal, read_name, read_symbol, write_decimal, write_name, write_symbol,
};
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

pub trait TokenTrait {
    fn initialize(e: Env, admin: Address, decimal: u32, name: Bytes, symbol: Bytes);

    fn allowance(e: Env, from: Address, spender: Address) -> i128;

    fn incr_allow(e: Env, from: Address, spender: Address, amount: i128);

    fn decr_allow(e: Env, from: Address, spender: Address, amount: i128);

    fn balance(e: Env, id: Address) -> i128;

    fn spendable(e: Env, id: Address) -> i128;

    fn authorized(e: Env, id: Address) -> bool;

    fn transfer(e: Env, from: Address, to: Address, amount: i128);

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn burn(e: Env, from: Address, amount: i128);

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);

    fn clawback(e: Env, admin: Address, from: Address, amount: i128);

    fn set_auth(e: Env, admin: Address, id: Address, authorize: bool);

    fn mint(e: Env, admin: Address, to: Address, amount: i128);

    fn set_admin(e: Env, admin: Address, new_admin: Address);

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> Bytes;

    fn symbol(e: Env) -> Bytes;
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

#[contract]
pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn initialize(e: Env, admin: Address, decimal: u32, name: Bytes, symbol: Bytes) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);

        write_decimal(&e, u8::try_from(decimal).expect("Decimal must fit in a u8"));
        write_name(&e, name);
        write_symbol(&e, symbol);
    }

    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, &from, &spender)
    }

    fn incr_allow(e: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&e, &from, &spender);
        let new_allowance = allowance
            .checked_add(amount)
            .expect("Updated allowance doesn't fit in an i128");

        write_allowance(&e, &from, &spender, new_allowance);
        event::incr_allow(&e, &from, &spender, amount);
    }

    fn decr_allow(e: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&e, &from, &spender);
        if amount >= allowance {
            write_allowance(&e, &from, &spender, 0);
        } else {
            write_allowance(&e, &from, &spender, allowance - amount);
        }
        event::decr_allow(&e, &from, &spender, amount);
    }

    fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, &id)
    }

    fn spendable(e: Env, id: Address) -> i128 {
        read_balance(&e, &id)
    }

    fn authorized(e: Env, id: Address) -> bool {
        is_authorized(&e, &id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        //from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&e, &from, amount);
        receive_balance(&e, &to, amount);
        event::transfer(&e, &from, &to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&e, &from, &spender, amount);
        spend_balance(&e, &from, amount);
        receive_balance(&e, &to, amount);
        event::transfer(&e, &from, &to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&e, &from, amount);
        event::burn(&e, &from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&e, &from, &spender, amount);
        spend_balance(&e, &from, amount);
        event::burn(&e, &from, amount)
    }

    fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_nonnegative_amount(amount);
        check_admin(&e, &admin);
        admin.require_auth();
        spend_balance(&e, &from, amount);
        event::clawback(&e, &admin, &from, amount);
    }

    fn set_auth(e: Env, admin: Address, id: Address, authorize: bool) {
        check_admin(&e, &admin);
        admin.require_auth();
        write_authorization(&e, &id, authorize);
        event::set_auth(&e, &admin, &id, authorize);
    }

    fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        check_admin(&e, &admin);
        admin.require_auth();
        receive_balance(&e, &to, amount);
        event::mint(&e, &admin, &to, amount);
    }

    fn set_admin(e: Env, admin: Address, new_admin: Address) {
        check_admin(&e, &admin);
        admin.require_auth();
        write_administrator(&e, &new_admin);
        event::set_admin(&e, &admin, &new_admin);
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> Bytes {
        read_name(&e)
    }

    fn symbol(e: Env) -> Bytes {
        read_symbol(&e)
    }
}
