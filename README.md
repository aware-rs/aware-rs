# aware - AWS Resource Explorer

```
$ aware ec2 --region us-west-1

vpc-05b9eed0a3a8f21f6 (demo-vpc)
├─ Subnets
│  ├─ subnet-0a01836ccc1a6ce32 (demo-subnet-private-us-west-1b)
│  ├─ subnet-03a3637827286bbf8 (demo-subnet-public-us-west-1c)
│  ├─ subnet-01a46bc386d553b32 (demo-subnet-public-us-west-1b)
│  └─ subnet-02681808f7350a4e4 (demo-subnet-private-us-west-1c)
├─ Instances
│  ├─ i-056ed593f5809e804 (demo-control-plane-tpr7s)
│  ├─ i-0f813195b9310d568 (demo-md-0-gnl76)
│  ├─ i-037ab5cfee8681939 (demo-md-0-7kknm)
│  ├─ i-01ddba6377d673f4e (demo-control-plane-7w97k)
│  ├─ i-09f3687c317efcc47 (demo-control-plane-ncqvs)
│  └─ i-0b70d9f40e150dc5b (demo-md-0-6pthq)
├─ Internet Gateways
│  └─ igw-02b58731d6bc1374a (demo-igw)
├─ Route Tables
│  ├─ rtb-0e8516b645cf8e4d2
│  ├─ rtb-0171c23f04ef0dac0 (demo-rt-public-us-west-1b)
│  ├─ rtb-0adaf4697a0c25b4d (demo-rt-private-us-west-1c)
│  ├─ rtb-01ceaba1226721144 (demo-rt-private-us-west-1b)
│  └─ rtb-0d9abf3fd86396940 (demo-rt-public-us-west-1c)
├─ Network ACLs
│  └─ acl-036e3d3e227b2c81d
├─ NAT Gateways
│  ├─ nat-0f3c01698a79e0092 (demo-nat)
│  └─ nat-03298f16b9784b057 (demo-nat)
├─ Security Groups
│  ├─ sg-053a1c528c4d70c47 (demo-lb)
│  ├─ sg-0679e14357bbfebbe (demo-controlplane)
│  ├─ sg-078dd7af27a22addc (demo-bastion)
│  ├─ sg-0a435fa0ff335704b (demo-node)
│  ├─ sg-0d1defcbbabd37659 (demo-apiserver-lb)
│  └─ sg-0f36b6950d359bc33 (default VPC security group)
└─ Network Interfaces
   ├─ eni-0b9d9cfbf812d0070 ()
   ├─ eni-095bbba49d7248680 ()
   ├─ eni-00e0d02aedceed6e5 (ELB demo-apiserver)
   ├─ eni-04f970021fecf7c24 (Interface for NAT Gateway nat-0f3c01698a79e0092)
   ├─ eni-04da26a35151518c8 ()
   ├─ eni-0f576bf6ea466f82b ()
   ├─ eni-0c5fd00e1dc69e5fe ()
   ├─ eni-0a3058f62bad1bc41 (ELB demo-apiserver)
   ├─ eni-0b8e0322dda50e046 (Interface for NAT Gateway nat-03298f16b9784b057)
   └─ eni-09b39574ed69388cf ()
```
