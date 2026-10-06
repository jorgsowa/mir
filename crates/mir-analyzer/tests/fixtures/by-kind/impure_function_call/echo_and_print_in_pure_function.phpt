===description===
@pure implies no side effects at all, but echo/print had no purity check
anywhere — a @pure function could freely write to the response body
unchecked.
===file===
<?php
/** @pure */
function usesEcho(): void {
    echo "side effect";
//  ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function echo() in a @pure function
}

/** @pure */
function usesPrint(): void {
    print "side effect";
//  ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function print() in a @pure function
}
