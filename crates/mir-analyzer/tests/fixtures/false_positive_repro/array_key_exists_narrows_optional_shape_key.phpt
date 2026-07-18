===description===
array_key_exists() clears `optional` but must not strip `null` — it proves
key presence, not a non-null value.
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
        if ($email === null) {
            return 'unknown';
        }
        /** @mir-check $email is string */
        return $email;
    }
    return "no email";
}
===expect===
