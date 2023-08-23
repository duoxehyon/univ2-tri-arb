// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

interface IUniswapV2Pair {
    function balanceOf(address owner) external view returns (uint);
    function approve(address spender, uint value) external returns (bool);
    function transfer(address to, uint value) external returns (bool);
    function transferFrom(address from, address to, uint value) external returns (bool);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function sync() external;
    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
}


contract TaxChecker {

    // AHHHH STACK OVERFLOW
    uint outOfInternal;
    uint feesInternal;

    function CheckTax(
        address tokenIn, // Token In
        address pair,    // Pair to check tax of
        uint    outOf,   // to divide with eg: 1000
        uint    fees     // fees, 997 for uniswap v2
    ) public returns(uint, uint) { // (Buy tax, Sell Tax)
        address _pair = pair;
        
        //Pprevent stack too deep errors
        outOfInternal = outOf;
        feesInternal = fees;
        
        // Get initial balance    
        (uint reserve0, uint reserve1,) = IUniswapV2Pair(_pair).getReserves();

        
        if (tokenIn == IUniswapV2Pair(_pair).token1()) {
            (reserve0, reserve1) = (reserve1, reserve0);
        }

        IUniswapV2Pair(tokenIn).transferFrom(_pair, address(this), reserve0 / 100);
        IUniswapV2Pair(_pair).sync();

        (reserve0, reserve1,) = IUniswapV2Pair(_pair).getReserves();


        address token0 = tokenIn;
        address token1;

        bool swapInDefault = true; // if 0 then token0 == token 

        uint16 taxIn;
        uint16 taxOut;

        if (tokenIn == IUniswapV2Pair(_pair).token1()) {
            (reserve0, reserve1) = (reserve1, reserve0);
            token1 =  IUniswapV2Pair(_pair).token0();
            swapInDefault = false;
        } else {
            token1 = IUniswapV2Pair(_pair).token1();
        }

        // Buy Tax
        {
            uint amountIn = IUniswapV2Pair(tokenIn).balanceOf(address(this));
            // Calc amountOut and transfer in amountIn to pair

            uint amountOut = getAmountOut(amountIn, reserve0, reserve1);
            uint amountOutExpected = amountOut;
            IUniswapV2Pair(token0).transfer(_pair, amountIn);

            // Check if this transfer was taxed
            if (IUniswapV2Pair(token0).balanceOf(_pair) - reserve0 != amountIn) {
                // If yes then re calculate amountOut
                    amountOut = getAmountOut((IUniswapV2Pair(token0).balanceOf(_pair) - reserve0), reserve0, reserve1);
            }

            // Do Swap
            amountOut -= 5; // Prevent Rounding error
            if (swapInDefault) {
                IUniswapV2Pair(_pair).swap(0 , amountOut, address(this), bytes(""));
            } else { 
                IUniswapV2Pair(_pair).swap(amountOut , 0, address(this), bytes(""));
            }

            uint difference = (amountOutExpected - IUniswapV2Pair(token1).balanceOf(address(this)));
            if (difference == 0) {
                taxIn = 0;
            } else {
                taxIn = uint16((difference * 10000) / amountOutExpected);
            }

        }

        // Update reserves
        (reserve0, reserve1,) = IUniswapV2Pair(_pair).getReserves();
        if (!swapInDefault) {
            (reserve0, reserve1) = (reserve1, reserve0);
        }

        // Sell Tax
        {
            // Get initial balance
            uint amountIn = IUniswapV2Pair(token1).balanceOf(address(this));

            // Calc amountOut and transfer in amountIn to pair
            uint amountOut = getAmountOut(amountIn, reserve1, reserve0);
            uint amountOutExpected = amountOut;
            IUniswapV2Pair(token1).transfer(_pair, amountIn);

            // Check if this transfer was taxed
                if (IUniswapV2Pair(token1).balanceOf(_pair) - reserve1 != amountIn) {
                    amountOut = getAmountOut((IUniswapV2Pair(token1).balanceOf(_pair) - reserve1), reserve1, reserve0);
            }

            // Do Swap
            amountOut -= 5; // Prevent Rounding error
            if (swapInDefault) {
                IUniswapV2Pair(_pair).swap(amountOut, 0, address(this), bytes(""));
            } else { 
                IUniswapV2Pair(_pair).swap(0 , amountOut, address(this), bytes(""));
            }
            
            uint difference = (amountOutExpected - IUniswapV2Pair(token0).balanceOf(address(this)));
            if (difference == 0) {
                taxOut = 0;
            } else {
                taxOut = uint16((difference * 10000) / amountOutExpected);
            }
        }

        return (taxIn, taxOut);
        
    }

    function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) internal view returns (uint) {
        uint amountInWithFee = amountIn * feesInternal;
        return amountInWithFee * reserveOut / ((reserveIn * outOfInternal) + amountInWithFee) + 1;
    
    }
}
