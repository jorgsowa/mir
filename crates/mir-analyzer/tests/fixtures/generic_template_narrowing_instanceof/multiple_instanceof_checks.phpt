===description===
multiple instanceof checks with OR logic
===file===
<?php
class Admin {
    public string $role = "admin";
}

class User {
    public string $role = "user";
}

class Guest {}

/**
 * @template TPerson as Admin|User|Guest
 * @param TPerson $person
 */
function hasRole(Admin|User|Guest $person): void {
    if ($person instanceof Admin || $person instanceof User) {
        echo $person->role;
    }
}

/**
 * Negated instanceof should narrow the opposite way
 * @template TPerson as Admin|User|Guest
 * @param TPerson $person
 */
function notGuest(Admin|User|Guest $person): void {
    if (!($person instanceof Guest)) {
        echo $person->role;
    }
}
===expect===
