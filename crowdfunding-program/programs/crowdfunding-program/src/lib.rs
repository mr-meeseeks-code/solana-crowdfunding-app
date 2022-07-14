use anchor_lang::prelude::*;

declare_id!("7vnf4qketJbYnbiEapQie7NjWqWtLocruCWX1TaUqe9x");

#[program]
pub mod crowdfunding_program {
    use super::*;

    pub fn create(ctx: Context<Create>, name: String, description: String, target_amount: u64) -> Result<()> {
        let compaing = &mut ctx.accounts.compaing;
        compaing.name = name;
        compaing.description = description;
        compaing.amount_donated = 0;
        compaing.target_amount = target_amount;
        // * - means dereferencing
        compaing.owner = *ctx.accounts.user.key;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let compaing = &mut ctx.accounts.compaing;
        let user = &mut ctx.accounts.user;
        if compaing.owner != *user.key {
            return Err(ErrorCode::InvalidOwner.into());
        }
        // Rent balance depends on data size
        let rent_balance = Rent::get()?.minimum_balance(compaing.to_account_info().data_len());
        if **compaing.to_account_info().lamports.borrow() - rent_balance < amount {
            return Err(ErrorCode::InvalidWithdrawAmount.into());
        }
        **compaing.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    } 

    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        let instruction = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.compaing.key(),
            amount
        );
        anchor_lang::solana_program::program::invoke(
            &instruction,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.compaing.to_account_info(),
            ]
        );
        let compaing = &mut ctx.accounts.compaing;
        compaing.amount_donated += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    // init means to create compaing account
    // bump to use unique address for compaing account
    #[account(init, payer=user, space=9000, seeds=[b"compaing_demo".as_ref(), user.key().as_ref()], bump)]
    pub compaing: Account<'info, Compaing>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub compaing: Account<'info, Compaing>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub compaing: Account<'info, Compaing>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Compaing {
    pub owner: Pubkey,
    pub name: String,
    pub description: String,
    pub amount_donated: u64,
    pub target_amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The user is not the owner of the compaing.")]
    InvalidOwner,
    #[msg("Insufficient amount to withdraw.")]
    InvalidWithdrawAmount,
}