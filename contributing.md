
# Sup

***This entire file is written by and from the perspective of DocJade.***

Thanks for wanting to help work on Doints!

To keep things clean, I ask that you read this whole document.

### No seriously. Read the guidelines

If you ignore them, your ideas, code, and all of your (hardly) work will be discarded. RTFM!

# What can/should I contribute?

Anything really!
- Ideas
- Opening issues
- Refactoring
- etc

If you have an idea, open a feature request for it! This doesn't mean it'll get worked on or even implemented, but ideas are the soul of innovation.

See an open issue, and think you can tackle it? Go for it!
Have an idea? Write it down!
Think something works in a stupid way? Talk is cheap! Send patches!

# What SHOULDN'T I do?

### Don't try to do things you don't understand.

IE, if you wanna contribute a new epic feature, but need new database stuff to get it done, you shouldn't try to implement it unless you know how to fully implement your idea, including that database stuff.

If you think something is out of your ability, you can sure as hell try (in fact, you SHOULD! This is how you learn things), but don't submit code that you know has flaws, or is _really_ jank. Some jank is fine tho.

### Tell people what you're working on

Please don't just open pull requests for new features without first making a tracking issue. Chances are, it wont be merged.

If you plan to work on a bug, claim it! Don't work on it silently!
# Ground Rules
### Etiquette

Don't be an asshole. Act in the repo like you'd act in Doccord.

### Cheating code

If you submit a pull request for shit like `make Weast always win coinflips` it will be rejected and I will be mad. Don't do it.

If you try to sneak in code to benefit yourself or others, the entire pull request will be rejected. Not just the section where you snuck that in, the ENTIRE pull.

### Shitty code

If you write some true garbage, it's gonna take a while to get it into a workable state.

Please try to keep things clean.

### Nesting

Become a Never Nester!
https://www.youtube.com/watch?v=CFRhGnuXG-4

If you're nesting for no reason, it makes code more complicated to read and write. My monitor is only so wide!

### Transactions

If you are doing ANYTHING related to moving doints around, or touching the database AT ALL, those actions MUST be wrapped in a transaction, if things fail, we need to roll back!

Thus, you should check all of your conditions on users (ie validating that they have enough doints for an action) either before, or during the transaction block.

### Testing

Testing isn't really required in most cases, but its a definite bonus if you can write tests for the code you're developing. Try some Test Driven Development!

But, I do ask that if you're creating bugfixes, if they rely solely on code that doesn't touch the DB or Discord, you should really try to write a test for that bug first, then fix that test case, so we can spot regressions in the future.

If you're able to abstract out logic that was previously coupled to either the DB or Discord, feel free to do so!

Some tests require certain environments, IE, there are some tests that use the DB for test transactions. If you don't have a local instance to test off of, don't worry about making those tests pass, I'll be checking them before merge.

### The house always wins

Actions should always have a negative expected value for the user, even if small. The bank needs to collect fees and taxes to keep things running.

For example see the `Who wants to be a Dillionare` slot machine, it has a house edge of 3%.

Things that are pure chance, like coin flips, get their house edge via charging transactional fees.

# Code style

### Formatting

Follow the Rust formatting conventions (clippy should yell at you if you ignore them) and run `cargo format` on your finalized code. Keeping things consistent makes Doints easy to work on.
### No vibe coding

FULLY AI generated code will not be merged. Doints are grass-fed and whole-grain. You are allowed to use AI for inspiration and debugging, but please don't submit entire swaths of code that was LLLM'ed into existence.

I expect you to fully understand how the code you are submitting works, if you get there with some help from `@grok is this true` that's fine, but you still need to hold that code up to the standards of the repo.

### Abstraction hell

My personal coding style prefers a lot of abstractions. If possible, you should be able to work on things at a very high level. IE, a new feature should be a call or two, and not require a whole bunch of handling by higher up.

IE if you made a new feature for, lets say a new slot machine, it shouldn't require changes outside of the Casino section if possible.

Raw function calls that span files (ie calling user_get_bal) should not be used, but instead you should use (or build if it doesnt exist) and interface for calling it. See the `BankInterface`, empty structs with methods are great!

When in doubt, create an interface to `Impl` against, or `Impl` against something that already exists!

### Way too many comments

It's better to have too many comments than not enough. You'll definitely notice the sheer amount of comments I write, and I expect the same level of code commenting from you. I need to be able to easily figure out what a section of code goes.

***CODE IS ___NOT___ SELF DOCUMENTING in 90% OF CASES!!!***

Yes I obviously don't want comments like
```rust
// Declare a varaible for the user that we found when we went looking for a user and found them.
// Thus with the found user we can go use that user to do other things.
// If we don't find the user,
// we bail out of the function because its expected to have the user at this level.
let found_user: User = helper_function_that_finds_a_user_that_meets_our_criteria(abc)?;

// Now that we have the user,
// we can foo on their bar so hard that they gudeon on my pindle till i hinge
found_user.foo(); // Foo them
```

But I do want comments like:
```rust
// Fees cannot be enabled if the sender is the bank
if transfer.sender == DointTransferParty::Bank && transfer.apply_fees {
    return Err(DointTransferError::TransferFeesOnBank)
}
```

Sure, its not too hard to deduce what this if statement does, but it lifts a massive burden off of maintainers (me) when I don't have to re-deduce an entire codebase all of the time.

I'm also a fan of separating sections of files (or even sections of code) with large, visible comments.
```rust
//
// //
// End of checks
// //
//
```

See [slots](src/invocable/standard/casino/slots.rs) for an example of the kind of comments I like, and documentation comments for features.

Explain things!

### Minimal pull requests

Sure, fixing a lot of bugs is cool, but I don't wanna have to look at changes in 10 different files. Please try to keep a sensible amount of things in a pull request. You don't _need_ to only put one issue per pull request, but please use common sense.

### To do, or not `todo!()`

As of right now, there's quite a few `//TODO: blah` comments scattered about. I'm working on it.

BUT, you should NOT add `//TODO:` comments unless they are blocked by another issue that is open. Why submit half finished code?

This also applies to `todo!()`

### Don't, or DO panic!

Its a discord bot at the end of the day. If one of your slash commands can panic under really weird circumstances, that's fine. Don't sweat it too hard.

What this DOESN'T mean is you can go willy-nilly with `.unwrap()` everywhere with no explanation, DOUBLY so for `.unwrap()`.

Even really weird situations should still return an error type if possible, not just explode.

### Respect our Clippy overlord

Having `Clippy::All` and `Clippy::Pedantic` may seem a bit extreme, but it really does help to make the code stink less. Don't use `#[allow(clippy::lint)` without a good explanation as to why its there. Comments!!!!!!

# Your First Contribution

What should you do first?

***Look at the easy issue tag!***

Otherwise?

Well, ask questions!
If you don't know how something works, or want an explanation of a feature (or even some rust syntax!) tell me!

Wanna work on something but the issue doesn't have enough info? Ask questions!

Try fixing an issue, optimizing something, cleaning up a loop, anything you can think of!

# Getting started
### Things you need

- Rust
	- I'm sure you can figure that out. If you can't, go away.
- Caffeine
	- The blue and orange redbulls are good

If you wanna do testing on DB related things, you'll need a MySql DB to work against, as that is the only platform that Doint bot works with.

### Things you can't have

- Database access
	- Besides user info privacy, I don't know of a safe way to share the DB even in a read-only way to let people look at it.
- The bot's discord token
	- Duh. That's mine.
	- Since Doint bot is for doccord, spinning up your own bot for testing would take some work, but if you're committed, you can figure it out. Do NOT selfbot in doccord to test changes if you decide to do that. Use your own server.
# How to report a bug

### Template:

You don't need to follow this template, but a good starting point is:

```
When I X, Y happens.
This is unexpected because REASON.
What should happen is THIS.
This can/cant be exploited for doint gain.
```

Don't mark the bug with any tags besides `bug`.
# How to suggest new ideas
### Template:

```
(Elevator pitch)

(What it would add)

(Why it should be added)

(What work it would take)

(What is blocking this idea, if any)
```

Make sure to mark the issue as a idea!
# Code review process

### Big Drother is watching.

I personally will review all code changes, I'm the final sign off point for the foreseeable future.

Submit a pull request. If I don't see it after a while, ping me.

# Commit / pull request format

There's no real requirements for this, I don't even care that much about putting `feat` into them. But please at least tag what issues a pull/commit fixes/implements.

# Thanks!
\- Doc
