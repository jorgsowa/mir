===description===
`array_key_exists('k', $arr)` proving an optional/nullable shape key present
must clear the key's `optional` flag and strip `null` from its type, the
same way `isset($arr['k'])` narrowing already does — add_key_to_sealed_shapes
only handled a key that was entirely absent from the shape (adding it as
`mixed`); a key that was already declared but `optional: true` fell through
unchanged, leaving later reads nullable/optional despite the proven guard.
===config===
suppress=MixedAssignment
===file===
<?php
/** @param array{name?: string} $data */
function greet(array $data): string {
    if (array_key_exists('name', $data)) {
        return $data['name'];
    }
    return "unknown";
}

/** @param array{email: string|null} $data */
function contact(array $data): string {
    if (array_key_exists('email', $data)) {
        $email = $data['email'];
        /** @mir-check $email is string */
        return $email;
    }
    return "no email";
}
===expect===
