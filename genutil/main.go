package main

import (
	"fmt"
	"math/big"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	coretypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/params"
)

func ChainConfig(chainType ChainType) *params.ChainConfig {
	return &params.ChainConfig{
		ChainID:                 getChainID(chainType),
		HomesteadBlock:          big.NewInt(0),
		DAOForkBlock:            nil,
		DAOForkSupport:          true,
		EIP150Block:             big.NewInt(0),
		EIP155Block:             big.NewInt(0),
		EIP158Block:             big.NewInt(0),
		ByzantiumBlock:          big.NewInt(0),
		ConstantinopleBlock:     big.NewInt(0),
		PetersburgBlock:         big.NewInt(0),
		IstanbulBlock:           big.NewInt(0),
		MuirGlacierBlock:        big.NewInt(0),
		BerlinBlock:             big.NewInt(0),
		LondonBlock:             big.NewInt(0),
		ArrowGlacierBlock:       nil,
		GrayGlacierBlock:        nil,
		TerminalTotalDifficulty: big.NewInt(0),
		MergeNetsplitBlock:      big.NewInt(0),
		ShanghaiTime:            newUint64(0),
		CancunTime:              newUint64(0),
		PragueTime:              newUint64(0),
		BlobScheduleConfig: &params.BlobScheduleConfig{
			Cancun: params.DefaultCancunBlobConfig,
			Prague: params.DefaultPragueBlobConfig,
		},
	}
}

// DefaultDevnetGenesisBlock returns the Devnet network genesis block.
func DefaultDevnetGenesisBlock() *core.Genesis {
	return &core.Genesis{
		Config:     ChainConfig(Devnet),
		Nonce:      0x1234,
		GasLimit:   60_000_000,
		Difficulty: big.NewInt(0x01),
		Timestamp:  1758542400,
		Alloc: coretypes.GenesisAlloc{
			common.HexToAddress("0xFC28736049E1ea4A315bFc4CfC6e09240250dfdf"): coretypes.Account{
				Code:    nil,
				Storage: nil,
				Balance: new(big.Int).Mul(big.NewInt(1_000_000_000), DECIMALS),
				Nonce:   0,
			},
		},
	}
}

func main() {
	var genesis *core.Genesis

	switch strings.ToLower(os.Args[1]) {
	case "devnet":
		genesis = DefaultDevnetGenesisBlock()
	default:
		panic(fmt.Errorf("invalid chain type: %s", os.Args[1]))
	}

	bz, _ := genesis.MarshalJSON()
	fmt.Println(string(bz))
}

//

var DECIMALS = new(big.Int).Exp(big.NewInt(10), big.NewInt(18), nil)

type ChainType int

const (
	Mainnet ChainType = iota
	Testnet
	Devnet
)

func getChainID(chainType ChainType) *big.Int {
	switch chainType {
	case Devnet:
		return big.NewInt(662532)
	case Testnet:
		return big.NewInt(662531)
	case Mainnet:
		return big.NewInt(66253)
	}

	panic(fmt.Sprintf("unknown chain type: %v", chainType))
}

func newUint64(val uint64) *uint64 { return &val }
