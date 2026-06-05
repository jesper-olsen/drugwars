// Drug Wars - A faithful Rust recreation of John E. Dell's 1984 DOS original
// Reconstructed from the DOS executable strings and TI-BASIC port logic.
// Copyright (1984) John E. Dell. This recreation is for preservation purposes.

use drugwars::Rng;
use std::fmt;
use std::io::{self, Write};
use std::ops::RangeInclusive;
use std::ops::{Index, IndexMut};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── CONSTANTS ───────────────────────────────────────────────────────────────
#[derive(Clone, Copy)]
enum Drug {
    Cocain = 0,
    Heroin,
    Acid,
    Weed,
    Speed,
    Ludes,
}

impl fmt::Display for Drug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Drug::Cocain => "COCAIN",
            Drug::Heroin => "HEROIN",
            Drug::Acid => "ACID",
            Drug::Weed => "WEED",
            Drug::Speed => "SPEED",
            Drug::Ludes => "LUDES",
        };
        f.pad(s)?;
        Ok(())
    }
}

const DRUGS: [Drug; 6] = [Cocain, Heroin, Acid, Weed, Speed, Ludes];
use Drug::*;

const DRUG_PRICES: [(i64, i64); DRUGS.len()] = [
    (15000, 30000),
    (5000, 14000),
    (1000, 4500),
    (300, 900),
    (70, 250),
    (10, 60),
];

#[derive(Clone, Copy, PartialEq)]
enum Location {
    Bronx = 0,
    Ghetto,
    CentralPark,
    Manhattan,
    ConeyIsland,
    Brooklyn,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Location::Bronx => "BRONX",
            Location::Ghetto => "GHETTO",
            Location::CentralPark => "CENTRAL PARK",
            Location::Manhattan => "MANHATTAN",
            Location::ConeyIsland => "CONEY ISLAND",
            Location::Brooklyn => "BROOKLYN",
        };
        f.pad(s)?;
        Ok(())
    }
}
const LOCATIONS: [Location; 6] = [
    Location::Bronx,
    Location::Ghetto,
    Location::CentralPark,
    Location::Manhattan,
    Location::ConeyIsland,
    Location::Brooklyn,
];

const GUNS: [&str; 4] = ["SATURDAY NIGHT SPECIAL", "RUGER", ".38 SPECIAL", "BARETTA"];

const GUN_COST: i64 = 400;
const GAME_DAYS: u32 = 30;
const TRENCH_UPGRADE_COST: i64 = 200;
const TRENCH_UPGRADE_SLOTS: i64 = 10;
const DOCTOR_COST: i64 = 1000;
const MAX_DAMAGE: i32 = 10;
const LOAN_INTEREST: f64 = 0.10;
const STARTING_CASH: i64 = 2000;
const STARTING_DEBT: i64 = 5500;
const MAX_BORROW: i64 = 5000;

// ─── STATE ───────────────────────────────────────────────────────────────────

struct DrugCounter([i64; DRUGS.len()]);

impl DrugCounter {
    fn total(&self) -> i64 {
        self.0.iter().sum()
    }
}

impl Default for DrugCounter {
    fn default() -> Self {
        DrugCounter([0; DRUGS.len()])
    }
}

impl Index<Drug> for DrugCounter {
    type Output = i64;
    fn index(&self, idx: Drug) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<Drug> for DrugCounter {
    fn index_mut(&mut self, idx: Drug) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

struct Game {
    cash: i64,
    bank: i64,
    debt: i64,
    day: u32,
    loc: Location,
    coat_size: i64,
    stash: DrugCounter,
    coat: DrugCounter,
    guns: u32,
    damage: i32,
    prices: DrugCounter,
    r: Rng,
}

impl Game {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        Game {
            cash: STARTING_CASH,
            bank: 0,
            debt: STARTING_DEBT,
            day: 1,
            loc: Location::Ghetto,
            coat_size: 100,
            stash: DrugCounter::default(),
            coat: DrugCounter::default(),
            guns: 0,
            damage: 0,
            prices: DrugCounter::default(),
            r: Rng::new(seed ^ 0xDEAD_BEEF_1984_CAFE),
        }
    }
    fn coat_free(&self) -> i64 {
        self.coat_size - self.coat.total()
    }

    pub fn net_worth(&self) -> i64 {
        // Stash drugs are abandoned at game end - not counted.
        self.cash + self.bank - self.debt
    }

    pub fn score(&self) -> u32 {
        let net = self.net_worth();
        if net <= 0 {
            0
        } else {
            ((net as f64 / 1_000_000.0 * 2.0).min(100.0)) as u32
        }
    }

    pub fn rank(&self) -> &'static str {
        match self.net_worth() {
            n if n >= 50_000_000 => "DRUG LORD",
            n if n >= 25_000_000 => "KINGPIN",
            n if n >= 5_000_000 => "BIG DEALER",
            n if n >= 1_000_000 => "DEALER",
            n if n >= 250_000 => "SMALL TIMER",
            n if n > 0 => "BAGBOY",
            _ => "BROKE JUNKIE",
        }
    }

    fn gen_prices(&mut self) {
        for (i, &(lo, hi)) in DRUG_PRICES.iter().enumerate() {
            self.prices.0[i] = self.r.range(lo, hi);
        }
    }

    pub fn buy_drug(&mut self, drug: Drug, amount: i64) -> Result<i64, &'static str> {
        let price = self.prices[drug];
        if self.coat_free() < amount {
            return Err("Not enough space in trench coat.");
        }
        if self.cash < amount * price {
            return Err("Not enough cash.");
        }

        let cost = amount * price;
        self.cash -= cost;
        self.coat[drug] += amount;
        Ok(cost)
    }

    pub fn sell_drug(&mut self, drug: Drug, amount: i64) -> Result<i64, &'static str> {
        if self.coat[drug] < amount {
            return Err("Not enough drug in trench coat.");
        }

        let earned = amount * self.prices[drug];
        self.cash += earned;
        self.coat[drug] -= amount;
        Ok(earned)
    }
}

// ─── I/O ─────────────────────────────────────────────────────────────────────
fn cls() {
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
}

fn pause() {
    print!("\n  (PRESS ENTER TO CONTINUE)");
    rl();
}

fn rl() -> String {
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim().to_uppercase()
}

fn get_int<T>(prompt: &str, range: RangeInclusive<T>) -> T
where
    T: std::str::FromStr,
    T: PartialOrd,
{
    loop {
        print!("  {prompt}: ");
        let _ = io::stdout().flush();

        if let Ok(n) = rl().parse::<T>()
            && range.contains(&n)
        {
            return n;
        }

        println!("  INVALID.");
    }
}

fn div() {
    println!("  ──────────────────────────────────────────────");
}

fn hdr(t: &str) {
    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║  {:40}║", t);
    println!("  ╚══════════════════════════════════════════╝");
    println!();
}

// ─── DISPLAY ─────────────────────────────────────────────────────────────────
fn show_status(g: &Game) {
    println!("  DAY {} / {}    LOCATION: {}", g.day, GAME_DAYS, g.loc);
    div();
    println!("  CASH  {:<14} BANK  {}", money(g.cash), money(g.bank));
    println!(
        "  DEBT  {:<14} GUNS  {}   DAMAGE {}/{MAX_DAMAGE}",
        money(g.debt),
        g.guns,
        g.damage
    );
    println!(
        "  HOLD  {}/{}  (FREE: {})",
        g.coat.total(),
        g.coat_size,
        g.coat_free()
    );
    div();
}
fn show_prices(g: &Game) {
    println!("  HEY DUDE, THE PRICES OF DRUGS HERE ARE:");
    println!();
    for drug in DRUGS {
        let price = money(g.prices[drug]);
        println!("    {drug:<12} {price:>8}");
    }
    println!();
}

fn money(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    let s = n.abs().to_string();

    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }

    format!("{sign}${}", out.chars().rev().collect::<String>())
}

fn show_coat(g: &Game) {
    println!("  TRENCH COAT:");
    println!();
    let mut any = false;
    for drug in DRUGS {
        if g.coat[drug] > 0 {
            println!("    {drug:<12} {}", g.coat[drug]);
            any = true;
        }
    }
    if !any {
        println!("    (empty)");
    }
    println!("    {:<12} {}", "FREE SPACE", g.coat_free());
    println!();
}
fn show_stash(g: &Game) {
    println!("  STASH (BRONX):");
    println!();
    let mut any = false;
    for drug in DRUGS {
        if g.stash[drug] > 0 {
            println!("    {drug:<12} {}", g.stash[drug]);
            any = true;
        }
    }
    if !any {
        println!("    (empty)");
    }
    println!();
}

// ─── RANDOM EVENTS ───────────────────────────────────────────────────────────
fn special(news: &str) {
    cls();
    hdr("!! SPECIAL BULLETIN !!");
    println!("{news}");
    pause();
}

fn events(g: &mut Game) -> bool {
    // returns true = dead
    let d = g.r.range(0, 20);
    match d {
        // 6 blanks out of 21
        1 => {
            special("  COPS MADE A BIG COKE BUST !!\n  PRICES ARE OUTRAGEOUS !!");
            g.prices[Drug::Cocain] = g.r.range(80_000, 140_000);
        }
        2 => {
            special(
                "  COLOMBIAN FREIGHTER DUSTED THE COAST GUARD !!\n  WEED PRICES HAVE BOTTOMED OUT !!",
            );
            g.prices[Drug::Weed] = g.r.range(40, 100);
        }
        3 => {
            special("  PIGS ARE SELLING CHEAP HEROIN FROM LAST WEEK'S RAID !!");
            g.prices[Drug::Heroin] = g.r.range(850, 2000);
        }
        4 => {
            special(
                "  RIVAL DRUG DEALERS RAIDED A PHARMACY AND ARE SELLING  C H E A P   L U D E S  !!!",
            );
            g.prices[Drug::Ludes] = g.r.range(2, 8);
        }
        5 => {
            special("  ADDICTS ARE BUYING HEROIN AT OUTRAGEOUS PRICES !!");
            g.prices[Drug::Heroin] = g.r.range(18_000, 43_000);
        }
        6 => {
            special("  THE MARKET HAS BEEN FLOODED WITH CHEAP HOME MADE ACID !!!");
            g.prices[Drug::Acid] = g.r.range(250, 800);
        }
        7 if g.coat.total() > 0 => {
            cls();
            hdr("!! BUSTED !!");
            let bl = g.r.range(2, 6);
            println!("  POLICE DOGS CHASE YOU FOR {bl} BLOCKS !!");
            let pct = g.r.range(10, 30);
            let mut dropped = false;
            for drug in DRUGS {
                if g.coat[drug] > 0 {
                    let d = g.coat[drug] * pct / 100;
                    g.coat[drug] = (g.coat[drug] - d).max(0);
                    if d > 0 {
                        dropped = true
                    }
                }
            }
            if dropped {
                println!("  YOU DROPPED SOME DRUGS !!  THAT'S A DRAG MAN !!");
            }
            pause();
        }
        8 if g.coat[Drug::Weed] > 0 || g.stash[Drug::Weed] > 0 => {
            cls();
            hdr("OH NO !!");
            println!("  YOUR MAMA MADE SOME BROWNIES AND USED YOUR WEED !!\n  THEY WERE GREAT !!");
            if g.coat[Drug::Weed] > 0 {
                g.coat[Drug::Weed] = 0;
            } else {
                g.stash[Drug::Weed] = 0;
            }
            pause();
        }
        9 => {
            cls();
            hdr("OH NO !!");
            let lost = g.cash / 3;
            g.cash -= lost;
            println!(
                "  YOU WERE MUGGED IN THE SUBWAY !!\n  YOU LOST {} !!",
                money(lost)
            );
            pause();
        }
        10 => {
            let free = g.coat_free();
            if free >= 3 {
                let amt = g.r.range(3, 8).min(free);
                let di = g.r.range(0, (DRUGS.len() - 1) as i64) as usize;
                let drug = DRUGS[di];
                cls();
                hdr("LUCKY FIND !!");
                println!("  YOU FIND {amt} UNITS OF {drug}");
                println!("  ON A DEAD DUDE IN THE SUBWAY !!");
                g.coat[drug] += amt;
                pause();
            }
        }
        11 => {
            cls();
            hdr("DANGER !!");
            println!("  THERE IS SOME WEED THAT SMELLS LIKE PARAQUAT HERE !!");
            println!("  IT LOOKS GOOD !!");
            print!("\n  WILL YOU SMOKE IT ? (Y/N): ");
            if rl().starts_with('Y') {
                cls();
                println!(
                    "  YOU HALUCINATE FOR THREE DAYS ON THE WILDEST TRIP YOU EVER IMAGINED !!!"
                );
                println!("\n  THEN YOU DIE BECAUSE YOUR BRAIN HAS DISINTEGRATED !!!");
                pause();
                return true;
            }
        }
        12 | 13 if g.cash >= TRENCH_UPGRADE_COST => {
            cls();
            hdr("OPPORTUNITY !!");
            println!("  WILL YOU BUY A NEW TRENCH COAT WITH MORE POCKETS");
            print!("  FOR {} ? (Y/N): ", money(TRENCH_UPGRADE_COST));
            if rl().starts_with('Y') {
                g.coat_size += TRENCH_UPGRADE_SLOTS;
                g.cash -= TRENCH_UPGRADE_COST;
                println!("  NEW COAT: {} POCKETS !", g.coat_size);
                pause();
            }
        }
        14 | 15 if g.cash >= GUN_COST => {
            let gi = g.r.range(0, GUNS.len() as i64 - 1) as usize;
            let gn = GUNS[gi];
            cls();
            hdr("OPPORTUNITY !!");
            println!("  WILL YOU BUY A {gn} FOR {} ?", money(GUN_COST));
            print!("  (Y/N): ");
            if rl().starts_with('Y') {
                g.guns += 1;
                g.cash -= GUN_COST;
                g.coat_size = (g.coat_size - 5).max(g.coat.total());
                println!("  YOU NOW HAVE {} GUN(S).", g.guns);
                pause();
            }
        }
        _ => {}
    }
    false
}

// ─── POLICE ENCOUNTER ────────────────────────────────────────────────────────
fn police(g: &mut Game) -> bool {
    // true => dead/arrested
    if g.coat.total() < 5 {
        return false;
    }
    let nd = g.r.range(1, 4) as u32;
    let mut cops = nd + 1;
    cls();
    hdr("POLICE ENCOUNTER !!");
    println!("  OFFICER HARDASS AND {nd} OF HIS DEPUTIES ARE CHASING YOU !!!!!");
    pause();

    loop {
        cls();
        div();
        println!("  DAMAGE  {} / {MAX_DAMAGE}", g.damage);
        println!("  COPS    {cops}");
        println!("  GUNS    {}", g.guns);
        div();
        println!();
        print!("  WILL YOU [R]UN OR [F]IGHT ? (R/F): ");
        let ch = rl();

        if ch.starts_with('R') {
            cls();
            if g.r.bool() {
                println!("  YOU LOST THEM IN THE ALLEYS !!");
                pause();
                offer_doctor(g);
                return false;
            } else {
                println!("  YOU CAN'T LOSE THEM !!");
                pause();
                if cops_shoot(g, cops) {
                    return true;
                }
            }
        } else if ch.starts_with('F') {
            cls();
            println!("  YOU'RE FIRING ON THEM !!!");
            if g.guns == 0 {
                println!("  BUT YOU DON'T HAVE ANY GUNS !!");
                pause();
                if cops_shoot(g, cops) {
                    return true;
                }
            } else if g.r.bool() {
                println!("  YOU KILLED ONE !!");
                cops -= 1;
                if cops == 0 {
                    let bounty = g.r.range(500, 2000);
                    g.cash += bounty;
                    cls();
                    println!("  YOU KILLED ALL OF THEM !!!!");
                    println!(
                        "\n  YOU FOUND {} ON OFFICER HARDASS' CARCAS !!!",
                        money(bounty)
                    );
                    pause();
                    offer_doctor(g);
                    return false;
                }
                pause();
                if cops_shoot(g, cops) {
                    return true;
                }
            } else {
                println!("  YOU MISSED THEM !!");
                pause();
                if cops_shoot(g, cops) {
                    return true;
                }
            }
        }
    }
}

fn offer_doctor(g: &mut Game) {
    if g.damage > 0 && g.cash >= DOCTOR_COST {
        cls();
        print!(
            "  WILL YOU PAY {} TO HAVE A DOCTOR SEW YOU UP ? (Y/N): ",
            money(DOCTOR_COST)
        );
        if rl().starts_with('Y') {
            g.cash -= DOCTOR_COST;
            g.damage = 0;
            println!("  THE DOCTOR PATCHES YOU UP !");
            pause();
        }
    }
}

// return true if dead
fn cops_shoot(g: &mut Game, cops: u32) -> bool {
    cls();
    println!("  THEY ARE FIRING ON YOU MAN !!");
    let hit = (0..cops).any(|_| g.r.bool());

    if hit {
        println!("  YOU'VE BEEN HIT !!");
        g.damage += 1;
        if g.damage >= MAX_DAMAGE {
            println!("\n  THEY WASTED YOU MAN !!!  WHAT A DRAG !!!");
            pause();
            return true;
        }
    } else {
        println!("  THEY MISSED !!");
    }
    pause();
    false
}

// ─── ACTIONS ─────────────────────────────────────────────────────────────────

fn buy(g: &mut Game) {
    cls();
    hdr("BUY DRUGS");
    show_prices(g);
    show_coat(g);
    if g.coat_free() == 0 {
        println!("  YOUR TRENCH COAT IS FULL !");
        pause();
        return;
    }
    println!("  WHAT WILL YOU BUY ?");
    for drug in DRUGS {
        let price = money(g.prices[drug]);
        println!("    {}. {drug:<12} {price:>8}", drug as usize + 1);
    }
    println!("    0. CANCEL");
    let n = get_int::<usize>("CHOICE", 0..=DRUGS.len());
    if n == 0 {
        return;
    }
    let drug = DRUGS[n - 1];
    let price = g.prices[drug];
    let max_buy = (g.cash / price).min(g.coat_free());
    println!(
        "\n  CAN AFFORD: {}  CAN HOLD: {}",
        g.cash / price,
        g.coat_free()
    );
    if max_buy == 0 {
        println!("  NOT ENOUGH CASH OR SPACE !");
        pause();
        return;
    }
    let amt = get_int::<i64>(&format!("HOW MUCH {drug} (max {max_buy})"), 0..=max_buy);
    if amt > 0 {
        let cost = g.buy_drug(drug, amt).unwrap();
        println!("  BOUGHT {amt} {drug} FOR {}.", money(cost));
        pause();
    }
}

fn sell(g: &mut Game) {
    cls();
    hdr("SELL DRUGS");
    show_prices(g);
    show_coat(g);
    if g.coat.total() == 0 {
        println!("  YOUR TRENCH COAT IS EMPTY !");
        pause();
        return;
    }
    println!("  WHAT WILL YOU SELL ?");
    for drug in DRUGS {
        if g.coat[drug] > 0 {
            println!(
                "    {}. {drug:<12} {} units @ ${} each",
                drug as usize + 1,
                g.coat[drug],
                g.prices[drug]
            );
        }
    }
    println!("    0. CANCEL");
    let n = get_int::<usize>("CHOICE", 0..=DRUGS.len());
    if n == 0 {
        return;
    }
    let drug = DRUGS[n - 1];
    let have = g.coat[drug];
    if have == 0 {
        println!("  YOU DON'T HAVE ANY {drug}.");
        pause();
        return;
    }
    let amt = get_int::<i64>(&format!("HOW MANY {drug} (max {have})"), 0..=have);
    if amt > 0 {
        let earned = g.sell_drug(drug, amt).unwrap();
        println!("  SOLD {amt} {drug} FOR {}.", money(earned));
        pause();
    }
}

fn jet(g: &mut Game) -> bool {
    // true => dead
    cls();
    hdr("JET - WHERE TO, DUDE ?");
    for loc in LOCATIONS {
        let here = if loc == g.loc { " <-- HERE" } else { "" };
        println!("    {}. {loc}{here}", loc as usize + 1);
    }
    println!("    0. STAY");
    println!();
    let n = get_int::<usize>("WHERE TO", 0..=LOCATIONS.len());
    if n == 0 {
        return false;
    }
    let dest = LOCATIONS[n - 1];
    if dest == g.loc {
        println!("  YOU'RE ALREADY THERE !");
        pause();
        return false;
    }
    println!("\n         . . .  S U B W A Y  . . .\n");
    g.loc = dest;
    g.day += 1;
    g.debt = (g.debt as f64 * (1.0 + LOAN_INTEREST)) as i64;
    g.gen_prices();
    if g.day <= GAME_DAYS {
        if events(g) {
            return true;
        }
        if g.coat.total() >= 5 && g.r.range(0, 3) == 0 && police(g) {
            return true;
        }
    }
    false
}

fn loan_shark(g: &mut Game) {
    if g.loc != Location::Bronx {
        cls();
        println!("  THE LOAN SHARK ONLY DEALS IN THE BRONX.");
        pause();
        return;
    }
    loop {
        cls();
        hdr("LOAN SHARK");
        println!("  YOUR DEBT  {}", money(g.debt));
        println!("  YOUR CASH  {}", money(g.cash));
        println!();
        println!("  1. REPAY DEBT\n  2. BORROW MORE\n  3. LEAVE");
        println!();
        match get_int::<usize>("CHOICE", 1..=3) {
            1 => {
                if g.debt == 0 {
                    println!("  NO DEBT !");
                    pause();
                    continue;
                }
                let mx = g.debt.min(g.cash);
                let amt = get_int::<i64>(&format!("REPAY HOW MUCH (max {})", money(mx)), 0..=mx);
                g.debt -= amt;
                g.cash -= amt;
                println!("  DEBT NOW: {}", money(g.debt));
                pause();
            }
            2 => {
                let amt = get_int::<i64>(
                    &format!("BORROW HOW MUCH (max {})", money(MAX_BORROW)),
                    0..=i64::MAX,
                );
                if amt == 0 || amt > MAX_BORROW {
                    println!("  YOU THINK HE IS CRAZY MAN !!!");
                } else {
                    g.debt += amt;
                    g.cash += amt;
                    println!("  BORROWED {}.  DEBT: {}", money(amt), money(g.debt));
                }
                pause();
            }
            _ => break,
        }
    }
}

fn bank(g: &mut Game) {
    if g.loc != Location::Bronx {
        cls();
        println!("  THE BANK IS IN THE BRONX.");
        pause();
        return;
    }
    loop {
        cls();
        hdr("BANK");
        println!("  ACCOUNT  {}", money(g.bank));
        println!("  CASH     {}", money(g.cash));
        println!();
        println!("  1. DEPOSIT\n  2. WITHDRAW\n  3. LEAVE");
        println!();
        match get_int::<usize>("CHOICE", 1..=3) {
            1 => {
                let amt = get_int::<i64>(
                    &format!("DEPOSIT HOW MUCH (max {})", money(g.cash)),
                    0..=g.cash,
                );
                g.bank += amt;
                g.cash -= amt;
                println!("  DEPOSITED {}.  ACCOUNT: {}", money(amt), money(g.bank));
                pause();
            }
            2 => {
                let amt = get_int::<i64>(
                    &format!("WITHDRAW HOW MUCH (max {})", money(g.bank)),
                    0..=g.bank,
                );
                g.bank -= amt;
                g.cash += amt;
                println!("  WITHDREW {}.  ACCOUNT: {}", money(amt), money(g.bank));
                pause();
            }
            _ => break,
        }
    }
}

fn stash_menu(g: &mut Game) {
    if g.loc != Location::Bronx {
        cls();
        println!("  YOUR STASH IS IN THE BRONX.");
        pause();
        return;
    }
    loop {
        cls();
        hdr("STASH");
        show_stash(g);
        show_coat(g);
        println!("  1. MOVE TO STASH\n  2. TAKE FROM STASH\n  3. LEAVE");
        println!();
        match get_int::<usize>("CHOICE", 1..=3) {
            1 => {
                println!("  WHICH DRUG TO STASH ?");
                for drug in DRUGS {
                    if g.coat[drug] > 0 {
                        println!("    {}. {drug} ({})", drug as usize + 1, g.coat[drug]);
                    }
                }
                let m = DRUGS.len();
                let msg = format!("DRUG (1-{m}, 0=cancel)");
                let n = get_int::<usize>(&msg, 0..=m);
                if n == 0 {
                    continue;
                }
                let drug = DRUGS[n - 1];
                if g.coat[drug] == 0 {
                    continue;
                }
                let mx = g.coat[drug];
                let amt = get_int::<i64>(&format!("HOW MANY (max {mx})"), 0..=mx);
                g.coat[drug] -= amt;
                g.stash[drug] += amt;
                println!("  MOVED {amt} {drug} TO STASH.");
                pause();
            }
            2 => {
                println!("  WHICH DRUG TO TAKE ?");
                for drug in DRUGS {
                    if g.stash[drug] > 0 {
                        println!("    {}. {drug} ({})", drug as usize + 1, g.stash[drug]);
                    }
                }
                let m = DRUGS.len();
                let msg = format!("DRUG (1-{m}, 0=cancel)");
                let n = get_int::<usize>(&msg, 0..=m);
                if n == 0 {
                    continue;
                }
                let drug = DRUGS[n - 1];
                if g.stash[drug] == 0 {
                    continue;
                }
                let mxt = g.stash[drug].min(g.coat_free());
                if mxt == 0 {
                    println!("  TRENCH COAT FULL !");
                    pause();
                    continue;
                }
                let amt = get_int::<i64>(&format!("HOW MANY (max {mxt})"), 0..=mxt);
                g.stash[drug] -= amt;
                g.coat[drug] += amt;
                println!("  MOVED {amt} {drug} TO COAT.");
                pause();
            }
            _ => break,
        }
    }
}

// ─── GAME OVER ────────────────────────────────────────────────────────────────
fn game_over(g: &Game, cause: &str) {
    cls();
    hdr("GAME OVER");
    println!("  {cause}");
    println!();

    println!("  FINAL TALLY:");
    div();
    println!("  CASH   {}", money(g.cash));
    println!("  BANK   {}", money(g.bank));
    println!("  DEBT   {}", money(g.debt));
    println!("  NET    {}", money(g.net_worth()));
    div();
    println!();
    println!("  CONGRATULATIONS !!");
    println!("  ON A SCALE OF 1 TO 100");
    println!("  YOUR RATING IS:  {}", g.score());
    println!();

    println!("  RANK: {}", g.rank());
    println!();
}

// ─── MAIN ────────────────────────────────────────────────────────────────────
fn play() {
    let mut g = Game::new();
    g.gen_prices();
    loop {
        if g.day > GAME_DAYS {
            game_over(&g, "YOUR MONTH IS UP !");
            return;
        }
        cls();
        show_status(&g);
        show_prices(&g);
        println!("  WHAT DO YOU WANT TO DO ?");
        println!();
        println!("    B - BUY DRUGS");
        println!("    S - SELL DRUGS");
        println!("    J - JET (TRAVEL)");
        if g.loc == Location::Bronx {
            println!("    L - LOAN SHARK");
            println!("    K - BANK");
            println!("    T - STASH");
        }
        println!("    C - VIEW TRENCH COAT");
        println!("    I - INSTRUCTIONS");
        println!("    Q - QUIT");
        println!();

        print!("  > ");
        match rl().as_str() {
            "B" => buy(&mut g),
            "S" => sell(&mut g),
            "J" if jet(&mut g) => {
                game_over(&g, "THEY WASTED YOU MAN !!!  WHAT A DRAG !!!");
                return;
            }
            "L" => loan_shark(&mut g),
            "K" => bank(&mut g),
            "T" => stash_menu(&mut g),
            "C" => {
                cls();
                hdr("TRENCH COAT");
                show_coat(&g);
                pause();
            }
            "I" => instructions(),
            "Q" => {
                game_over(&g, "YOU QUIT EARLY.");
                return;
            }
            _ => {}
        }
    }
}

fn title() {
    cls();
    println!();
    println!("  ╔═══════════════════════════════════════════════╗");
    println!("  ║                                               ║");
    println!("  ║            D R U G   W A R S                  ║");
    println!("  ║                                               ║");
    println!("  ║      A GAME BASED ON THE NEW YORK             ║");
    println!("  ║               DRUG MARKET                     ║");
    println!("  ║                                               ║");
    println!("  ║           BY JOHN E. DELL                     ║");
    println!("  ║            COPYRIGHT (1984)                   ║");
    println!("  ║                                               ║");
    println!("  ╚═══════════════════════════════════════════════╝");
    println!();
    println!();
}

fn instructions() {
    cls();
    hdr("INSTRUCTIONS");
    println!("  This is a game of buying, selling, and fighting.");
    println!("  The object of the game is to pay off your debt to");
    println!("  the loan shark.  Then, make as much money as you");
    println!("  can in a 1 month (30 day) period.  If you deal too");
    println!("  heavily in drugs, you might run into the police !!");
    println!();
    println!("  Your main drug stash will be in the Bronx.");
    println!("  (It's a nice neighborhood)");
    println!();
    println!("  The prices of drugs per unit are:");
    println!();
    println!("      COCAINE     15000-30000");
    println!("      HEROIN       5000-14000");
    println!("      ACID         1000-4500");
    println!("      WEED          300-900");
    println!("      SPEED          70-250");
    println!("      LUDES          10-60");
    println!();
    println!("  The Loan Shark and Bank are only available in the Bronx.");
    println!("  The Loan Shark charges 10% interest per day - pay him back!");
    println!();
    pause();
}

fn main() {
    loop {
        title();
        print!("  DO YOU WANT INSTRUCTIONS ? (Y/N): ");
        if rl().starts_with('Y') {
            instructions();
        }
        play();
        print!("\n  PLAY AGAIN ? (Y/N): ");
        if rl().starts_with('N') {
            cls();
            println!("  THANKS FOR PLAYING !\n");
            println!("  DRUG WARS  --  COPYRIGHT (1984) JOHN E. DELL");
            println!("  Rust port for preservation purposes.\n");
            break;
        }
    }
}
