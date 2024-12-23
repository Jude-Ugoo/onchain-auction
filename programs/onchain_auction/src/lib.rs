use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer};
use anchor_spl::associated_token::AssociatedToken;


declare_id!("AX5U2r9HWTtnt9Py2bGV86GEqDf4KAvw4axanZXa5gQn");

#[program]
pub mod onchain_auction {
    use super::*;

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        auction_id: u64,
        reserve_price: u64,
        start_time: i64,
        end_time: i64,
        bid_increment: u64,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;

        auction.auction_id = auction_id;
        auction.seller = *ctx.accounts.seller.key;
        auction.reserve_price = reserve_price;
        auction.highest_bidder = None;
        auction.highest_bid = reserve_price;
        auction.start_time = start_time;
        auction.end_time = end_time;
        auction.bid_increment = bid_increment;
        auction.is_active = true;

        Ok(())
    }

    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        let auction_data = &ctx.accounts.auction;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp >= auction_data.start_time, AuctionError::AuctionInactive);
        require!(clock.unix_timestamp < auction_data.end_time, AuctionError::AuctionEnded);
        require!(bid_amount >= auction_data.highest_bid + auction_data.bid_increment, AuctionError::BidTooLow);
        require!(auction_data.is_active, AuctionError::AuctionInactive);

        let auction = &mut ctx.accounts.auction; // Mutable borrow for updates

        if let Some(highest_bidder) = auction.highest_bidder {
            // Refund previous highest bidder
            let refund = Transfer {
                from: ctx.accounts.auction_ata.to_account_info(),
                to: ctx.accounts.previous_bidder_ata.to_account_info(),
                authority: ctx.accounts.auction.to_account_info(),
            };
            anchor_spl::token::transfer(CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                refund,
            ), auction.highest_bid)?;
        }

        // Transfer new bid
        let transfer_bid = Transfer {
            from: ctx.accounts.bidder_ata.to_account_info(),
            to: ctx.accounts.auction_ata.to_account_info(),
            authority: ctx.accounts.bidder.to_account_info(),
        };
        

        Ok(())
    }
    
    
    pub fn end_auction(ctx: Context<EndAuction>) -> Result<()> {
        
    
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(auction_id: u64)]
pub struct CreateAuction<'info> {
    #[account(
        init,
        payer = seller,
        seeds = [b"auction", auction_id.to_le_bytes().as_ref()],
        bump,
        space = 8 + 32 + 64 + 64 + 32 + 8 + 8 + 32 + 8 + 1
    )]
    pub auction: Account<'info, Auction>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut, has_one = auction_ata)]
    pub auction: Account<'info, Auction>,

    #[account(mut)]
    pub auction_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(
        mut, 
        constraint = bidder_ata.owner == bidder.key() && bidder_ata.mint == auction_ata.mint,
    )]
    pub bidder_ata: Account<'info, TokenAccount>,

    #[account(
        mut, 
        constraint = highest_bidder_ata.mint == auction_ata.mint @ AuctionError::InvalidMint
    )]
    pub highest_bidder_ata: Account<'info, TokenAccount>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub previous_bidder_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct EndAuction<'info> {
    #[account(mut, has_one = auction_ata)]
    pub auction: Account<'info, Auction>,

    #[account(mut)]
    pub auction_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub winner_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Auction {
    pub auction_id: u64,
    pub seller: Pubkey,
    pub reserve_price: u64,
    pub highest_bid: u64,
    pub highest_bidder: Option<Pubkey>,
    pub start_time: i64,
    pub end_time: i64,
    pub bid_increment: u64,
    pub is_active: bool,
}

#[error_code]
pub enum AuctionError {
    #[msg("Auction has already ended.")]
    AuctionEnded,
    #[msg("Bid is too low.")]
    BidTooLow,
    #[msg("Auction is inactive.")]
    AuctionInactive,
    #[msg("Auction has not ended yet.")]
    AuctionNotEnded,
    #[msg("Invalid mint for the token account.")]
    InvalidMint,
}
