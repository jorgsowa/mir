===description===
@pure implies no side effects at all, but echo/print had no purity check
anywhere — a @pure function could freely write to the response body
unchecked.
===file===
<?php
/** @pure */
function usesEcho(): void {
    echo "side effect";
}

/** @pure */
function usesPrint(): void {
    print "side effect";
}
===expect===
ImpureFunctionCall@4:4-4:23: Calling impure function echo() in a @pure function
ImpureFunctionCall@9:4-9:23: Calling impure function print() in a @pure function
