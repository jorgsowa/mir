===description===
`pcntl_signal_get_handler()` returns the installed callable or an int (`SIG_DFL`/`SIG_IGN`), so its result can be an array callable.
===file===
<?php
final class SignalRegistry {
    public function handle(int $signal): void {
        echo $signal;
    }
}

function ownsHandler(int $signal): bool {
    $previous = pcntl_signal_get_handler($signal);
    return is_array($previous) && $previous[0] instanceof SignalRegistry ? true : false;
}

function isDefault(int $signal): bool {
    $previous = pcntl_signal_get_handler($signal);
    if (is_int($previous)) {
        return $previous === \SIG_DFL;
    }
    return false;
}
