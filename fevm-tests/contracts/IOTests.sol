// SPDX-License-Identifier: Apache-2.0 MIT
pragma solidity ^0.8.17;

contract IOTests {
    function uints0in1out() public pure returns (uint32) {
        return 5;
    }

    function uints0in2out() public pure returns (uint32, uint32) {
        return (5, 10);
    }
    
    function uints0in3out() public pure returns (uint32, uint32, uint32) {
        return (5, 10, 15);
    }

    function strings0in1out() public pure returns (string memory) {
        return "1";
    }

    function strings0in2out() public pure returns (string memory, string memory) {
        return ("1", "5");
    }
    
    function strings0in3out() public pure returns (string memory, string memory, string memory) {
        return ("1", "5", "10");
    }

    function uints1in1out(uint32 a) public pure returns (uint32) {
        return a;
    }

    function uints2in2out(uint32 a, uint32 b) public pure returns (uint32, uint32) {
        return (a, b);
    }

    function uints2in1out(uint32 a, uint32 b) public pure returns (uint32) {
        return a + b;
    }

    function strings1in1out(string calldata a) public pure returns (string calldata) {
        return a;
    }

    function strings2in2out(string calldata a, string calldata b) public pure returns (string calldata, string calldata) {
        return (a, b);
    }

    function strings2in1out(string calldata a, string calldata b) public pure returns (string memory) {
        return string.concat(a, b);
    }
}
