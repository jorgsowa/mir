===description===
Sanity check that the negated enum-case narrowing fix actually narrows the
variable's type (not just incidentally silencing the match check
elsewhere): after excluding Status::Pending, $s is provably
Status::Active|Status::Inactive.
===config===
suppress=UnusedVariable
===file===
<?php
enum Status { case Active; case Inactive; case Pending; }

function foo(Status $s): void {
    if ($s === Status::Pending) {
        return;
    }
    /** @mir-check $s is Status::Active|Status::Inactive */
    $_ = 1;
}
===expect===
