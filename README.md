A schemey lisp

# Function calling notes
* Each function needs an activation record
* Details the function's storage needs
    * Parameters
    * Temporaries
    * Results
* Points to calling function record
    * Not sure why? Exceptions?
* Need to keep in mind dynamic scoping vs lexical scoping
    * Enclosing scopes are different

# Register Allocation Notes
* Liveness analysis
    * Interference graph
        * Constructed by understanding what registers are alive at any given point
        * Have to choose an appropriate scope - function? Lexical scope? Nested?
    * Graph coloring
        * Color graph with N colours, N == amount of registers 
        * Not all registers can be colored per gc rules
        * Uncolored registers are 'spilled' - relegated to stack storage
        * BUT the decision to spill needs to be weighted according to the cost of stacking the register

