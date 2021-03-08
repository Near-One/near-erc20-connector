pragma solidity 0.7.6;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract eNear is ERC20 {

    address public minterAndBurner;

    constructor(
        string memory _tokenName,
        string memory _tokenSymbol,
        address _minterAndBurner
    ) ERC20(_tokenName, _tokenSymbol) {
        require(_minterAndBurner != address(0), "Minter is invalid");
        minterAndBurner = _minterAndBurner;

        // set decimals to 24 to mirror yocto Near
        _setupDecimals(24);
    }

    // As Near max supply is 1,000,000,000 x 10 ^ 24, uint256 will be able to handle
    function mintTo(address _recipient, uint256 _amount) external {
        require(msg.sender == minterAndBurner, "mint: Only minter");
        require(_amount > 0, "mint: Invalid amount specified");
        _mint(_recipient, _amount);
    }

    // for the sender to burn their own tokens
    function burn(uint256 _amount) external {
        _burn(msg.sender, _amount);
    }
}
