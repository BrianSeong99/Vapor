// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title MockUSDC
 * @dev Simple mock USDC token for testing and demo purposes
 */
contract MockUSDC is ERC20, Ownable {
    uint8 private _decimals = 6;
    
    constructor() ERC20("Mock USD Coin", "USDC") Ownable(msg.sender) {
        // Mint initial supply for testing
        _mint(msg.sender, 1000000 * 10**_decimals); // 1M USDC
    }
    
    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }
    
    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
}
